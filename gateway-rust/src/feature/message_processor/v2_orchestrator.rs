use crate::AppState;
use crate::common::identity::UserIdentity;
use crate::common::repository::message_repo::save_message;
use crate::feature::conversation::model::MessageItem;
use crate::feature::message_processor::v2_agent_orchestrator::V2AgentOrchestrator;
use crate::feature::{
    Conversation, MessageSource, OutboundMessage, PresenceMessage, UnifiedMessage,
};
use crate::prompts::StatusRegistry;
use crate::services::event_dispatcher::AppEvent;
use anyhow::anyhow;
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

pub async fn process_v2_message(
    state: AppState,
    convo: Conversation,
    msg: UnifiedMessage,
) -> anyhow::Result<MessageItem> {
    let conversation_id = msg.conversation_id;
    let text_content = msg.text_content.clone();

    info!(
        conversation_id = %conversation_id,
        user_id = ?msg.user_id,
        source = ?msg.source,
        "Processing unified message v2"
    );

    // 0. Silent Media Buffer: Media with EMPTY text and NO trigger
    let has_media = msg.image_url.is_some()
        || msg.video_url.is_some()
        || msg.audio_url.is_some()
        || msg.doc_url.is_some()
        || msg.sticker_url.is_some();

    let text_trimmed = text_content.trim();
    let has_only_trigger = text_trimmed.to_lowercase() == "nom" || text_trimmed == "/cmd";
    let is_skip = text_trimmed.eq_ignore_ascii_case("skip");

    let members = sqlx::query!(
        "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or(Vec::new());

    // ======== Media Context Hydration (Multimodal Interpreter) ========//
    let mut hydrated_text = text_content.clone();
    let mut media_to_hydrate = Vec::new();
    if let Some(url) = &msg.image_url {
        media_to_hydrate.push((url, "image/jpeg"));
    }
    if let Some(url) = &msg.audio_url {
        media_to_hydrate.push((url, "audio/mpeg"));
    }
    if let Some(url) = &msg.video_url {
        media_to_hydrate.push((url, "video/mp4"));
    }
    if let Some(url) = &msg.doc_url {
        media_to_hydrate.push((url, "application/pdf"));
    }
    if let Some(url) = &msg.sticker_url {
        media_to_hydrate.push((url, "image/webp"));
    }

    if !media_to_hydrate.is_empty() {
        let interpreter = crate::services::media_interpreter::MediaInterpreterService;
        let boot_dispatcher = crate::common::tools::ToolDispatcher::new(
            state.pool.clone(),
            std::path::PathBuf::from("."),
            Some(conversation_id),
            msg.user_id,
            state.gemini.clone(),
            state.gemini_api_key.clone(),
            state.storage.clone(),
            state.clone(),
        );

        let mut descriptions = Vec::new();
        for (url, mime) in media_to_hydrate {
            match interpreter
                .hydrate_media_context_string(&boot_dispatcher, url, mime)
                .await
            {
                Ok((hydrated, _)) => {
                    // Extract the description from "[Media Context Description: <desc>] <original>"
                    if let Some(start) = hydrated.find("Description: ") {
                        if let Some(end) = hydrated.rfind("] ") {
                            descriptions.push(hydrated[start + 13..end].to_string());
                        }
                    }
                }
                Err(e) => error!("Media Interpreter: Failed to hydrate {}: {}", mime, e),
            }
        }

        if !descriptions.is_empty() {
            hydrated_text = format!(
                "[Media Context: {}] {}",
                descriptions.join(" | "),
                text_content
            );
        }
    }

    let mut quoted_metadata = None;
    if let Some(q) = &msg.quoted_message {
        let mut q_with_name = json!(q);

        // Attempt to look up the display name of the quoted sender from our database
        let quoted_sender_name: Option<String> = sqlx::query_scalar!(
            "SELECT u.display_name FROM users u JOIN channels c ON c.user_id = u.id WHERE c.external_id = $1 LIMIT 1",
            q.sender_id
        )
        .fetch_optional(&state.pool)
        .await
        .unwrap_or(None)
        .flatten();

        if let Some(name) = quoted_sender_name {
            if let Some(obj) = q_with_name.as_object_mut() {
                obj.insert("display_name".to_string(), json!(name));
            }
        }

        quoted_metadata = Some(json!({ "quoted_message": q_with_name }));
    }

    // 1. Immediate Save (Hydrated content for better history context)
    let save_user_message = save_message(
        &state.pool,
        conversation_id,
        "user",
        &hydrated_text, // Use the description from MediaInterpreter if available
        None,
        msg.user_id,
        0,
        0,
        0,
        msg.image_url.clone(),
        msg.video_url.clone(),
        msg.audio_url.clone(),
        msg.doc_url.clone(),
        msg.sticker_url.clone(),
        quoted_metadata,
    )
    .await;
    if let Err(e) = save_user_message {
        info!("Saving message failed :{}", e);
        return Err(anyhow!("Failed to save message {}", e));
    }

    let mut saved_message = save_user_message?;
    // start event
    //notify message incoming
    for member in members.iter().map(|v| v.user_id) {
        info!("notify user message saved :{:?}", member);
        saved_message.display_name = Some(msg.display_name.clone().unwrap());
        let _ = state
            .dispatch(AppEvent::user(
                member.to_string().as_str(),
                "message",
                saved_message.to_sse_json(0),
            ))
            .await;
    }
    // ======== Interaction Gate (Pre-Filtering for Groups) ========//
    if msg.is_group && !msg.is_mentioned {
        let gate = crate::services::interaction_gate::InteractionGateService::new(
            state.pool.clone(),
            state.gemini_api_key.clone(),
        );

        let should_respond = match gate
            .should_respond_to_group_message(&hydrated_text, false)
            .await
        {
            Ok(true) => {
                info!("Interaction Gate: Passed, continuing to process group message.");
                true
            }
            Ok(false) => {
                info!(
                    "Interaction Gate: Dropping ambient message in group {}",
                    msg.conversation_id
                );
                false
            }
            Err(e) => {
                error!(
                    "Interaction Gate: Error during evaluation: {}. Continuing as fallback.",
                    e
                );
                false
            }
        };

        if !should_respond {
            info!("Interaction Gate: Not Passed, save to group message, but no reply needed.");
            return Ok(saved_message);
        }
    }

    let guard_rail = crate::services::guardrail::GuardrailService::new(
        state.pool.clone(),
        state.gemini_api_key.clone(),
    );

    let is_injection = guard_rail
        .is_injection_detected(msg.text_content.as_str())
        .await
        .unwrap_or_else(|_| false);

    let orchestrator = V2AgentOrchestrator::new(
        state.clone(),
        Some(convo),
        Some(UserIdentity {
            id: msg.user_id.unwrap_or(Uuid::nil()),
            display_name: "".to_string(),
        }),
        members.iter().map(|v| v.user_id.clone()).collect(),
    );

    if is_injection {
        info!("Guardrail: Injection detected. Injecting rejection prompt.");

        let _ = orchestrator
            .process_v2_message_with_intent(
                state.clone(),
                msg,
                hydrated_text, // Pass the hydrated description instead of raw text
                Some(crate::prompts::PromptRegistry::guardrail_rejection().to_string()),
            )
            .await;
        return Ok(saved_message);
    }

    if has_media && has_only_trigger {
        let media_url = msg
            .image_url
            .as_ref()
            .or(msg.video_url.as_ref())
            .or(msg.audio_url.as_ref())
            .or(msg.doc_url.as_ref())
            .or(msg.sticker_url.as_ref())
            .unwrap();

        let media_type = if msg.image_url.is_some() {
            "image"
        } else if msg.video_url.is_some() {
            "video"
        } else if msg.audio_url.is_some() {
            "audio"
        } else if msg.doc_url.is_some() {
            "document"
        } else {
            "sticker"
        };

        info!(
            "Triggered media clarification (poke detected) for {}: {}",
            media_type, media_url
        );

        let injected_system_prompt =
            crate::prompts::PromptRegistry::zero_intent_clarification().to_string();

        let _ = orchestrator
            .process_v2_message_with_intent(
                state.clone(),
                msg,
                format!("[User poked about this {}]", media_type),
                Some(injected_system_prompt),
            )
            .await;
        Ok(saved_message)
    } else if has_media && !is_skip {
        // Task 1: If message.text is NOT empty: Do not ask for clarification.
        // Combine the text and the image into a single multi-part prompt for Gemini.
        let injected_system_prompt =
            crate::prompts::PromptRegistry::media_with_text_instruction().to_string();
        let _ = orchestrator
            .process_v2_message_with_intent(
                state.clone(),
                msg,
                hydrated_text, // Use the hydrated description here
                Some(injected_system_prompt),
            )
            .await;
        Ok(saved_message)
    } else {
        let _ = orchestrator
            .process_v2_message_with_intent(state.clone(), msg, hydrated_text, None)
            .await;

        Ok(saved_message)
    }
}

pub async fn send_status_update(
    state: &AppState,
    members: Vec<Uuid>,
    conversation_id: Uuid,
    source: MessageSource,
    is_group: bool,
    event: String,
    text: String,
) {
    info!("send_status_update start");
    let state = state.clone();
    let pool = state.pool.clone();
    let event = event.clone();
    tokio::spawn(async move {
        let convo = sqlx::query!(
            "SELECT conversation_type,id FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&pool)
        .await;

        let ch_names = match source.clone() {
            MessageSource::Web { name } => vec![name.to_string()],
            MessageSource::Telegram { name } => vec![name.to_string()],
            MessageSource::WhatsApp { name } => vec![name.to_string()],
            MessageSource::Other { name } => vec![name.to_string()],
            MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(data) = convo {
            if data.conversation_type.eq_ignore_ascii_case("private") {
                info!("send_status_update web");
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "text":text
                            }),
                        ))
                        .await;
                }

                if !is_group {
                    let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let outbound = OutboundMessage {
                            is_group,
                            sender_id: channel.external_id.clone(),
                            conversation_id: channel.external_chat_id.clone(),
                            text: text.clone(),
                            channel: channel.channel_type.clone(),
                            video_url: None,
                            image_url: None,
                            audio_url: None,
                            doc_url: None,
                            sticker_url: None,
                            metadata: None,
                        };
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    &event,
                                    json!({"text": text}),
                                )
                                .with_redis_outbound(outbound),
                            )
                            .await;
                    }
                }
            } else {
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "text":text
                            }),
                        ))
                        .await;
                }

                if !is_group {
                    info!("send_status_update channel:{:?}", ch_names);
                    let channel_info = sqlx::query!(
                        "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                        conversation_id,
                        &ch_names[..]
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let outbound = OutboundMessage {
                            is_group: false,
                            sender_id: "".to_string(),
                            conversation_id: channel.external_group_id.clone(),
                            text: text.clone(),
                            channel: channel.channel.clone(),
                            video_url: None,
                            image_url: None,
                            audio_url: None,
                            doc_url: None,
                            sticker_url: None,
                            metadata: None,
                        };
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    &event,
                                    json!({"text": text}),
                                )
                                .with_redis_outbound(outbound),
                            )
                            .await;
                    }
                }
            }
        }
    });
}

pub async fn send_tool_update(
    state: &AppState,
    members: Vec<Uuid>,
    conversation_id: Uuid,
    source: MessageSource,
    is_group: bool,
    event: String,
    tool_name: String,
) {
    let state = state.clone();
    let pool = state.pool.clone();
    let event = event.clone();
    tokio::spawn(async move {
        let convo = sqlx::query!(
            "SELECT conversation_type,id FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&pool)
        .await;

        let ch_names = match source.clone() {
            MessageSource::Web { name } => vec![name.to_string()],
            MessageSource::Telegram { name } => vec![name.to_string()],
            MessageSource::WhatsApp { name } => vec![name.to_string()],
            MessageSource::Other { name } => vec![name.to_string()],
            MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(data) = convo {
            if data.conversation_type.eq_ignore_ascii_case("private") {
                info!("send_status_update web");
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "name":tool_name,
                                "text":StatusRegistry::random_action_phrase(tool_name.as_str())
                            }),
                        ))
                        .await;
                }

                if !is_group {
                    let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let outbound = OutboundMessage {
                            is_group,
                            sender_id: channel.external_id.clone(),
                            conversation_id: channel.external_chat_id.clone(),
                            text: StatusRegistry::random_action_phrase(tool_name.as_str()),
                            channel: channel.channel_type.clone(),
                            video_url: None,
                            image_url: None,
                            audio_url: None,
                            doc_url: None,
                            sticker_url: None,
                            metadata: None,
                        };
                        let _ = state.dispatch(AppEvent::conversation(conversation_id, &event, json!({"name": tool_name, "text": StatusRegistry::random_action_phrase(tool_name.as_str())})).with_redis_outbound(outbound)).await;
                    }
                }
            } else {
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "name":tool_name,
                                "text":StatusRegistry::random_action_phrase(tool_name.as_str())
                            }),
                        ))
                        .await;
                }

                if !is_group {
                    info!("send_status_update channel:{:?}", ch_names);
                    let channel_info = sqlx::query!(
                        "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                        conversation_id,
                        &ch_names[..]
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let outbound = OutboundMessage {
                            is_group: false,
                            sender_id: "".to_string(),
                            conversation_id: channel.external_group_id.clone(),
                            text: tool_name.clone(),
                            channel: channel.channel.clone(),
                            video_url: None,
                            image_url: None,
                            audio_url: None,
                            doc_url: None,
                            sticker_url: None,
                            metadata: None,
                        };
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    &event,
                                    json!({"name": tool_name}),
                                )
                                .with_redis_outbound(outbound),
                            )
                            .await;
                    }
                }
            }
        }
    });
}

pub async fn send_status_presence_update(
    state: &AppState,
    members: Vec<Uuid>,
    conversation_id: Uuid,
    source: MessageSource,
    is_group: bool,
    is_typing: bool,
) {
    let state = state.clone();
    let pool = state.pool.clone();
    let event = "presence".to_string();
    tokio::spawn(async move {
        let convo = sqlx::query!(
            "SELECT conversation_type,id FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&pool)
        .await;

        let ch_names = match source.clone() {
            MessageSource::Web { name } => vec![name.to_string()],
            MessageSource::Telegram { name } => vec![name.to_string()],
            MessageSource::WhatsApp { name } => vec![name.to_string()],
            MessageSource::Other { name } => vec![name.to_string()],
            MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(data) = convo {
            if data.conversation_type.eq_ignore_ascii_case("private") {
                info!("send_status_update web");
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({"conversation_id": conversation_id,"is_typing": is_typing,"user_id": "nomi","text":""}),
                        ))
                        .await;
                }

                if !is_group {
                    let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let presence = PresenceMessage {
                            sender_id: channel.external_id.clone(),
                            chat_id: channel.external_chat_id.clone(),
                            channel: channel.channel_type.clone(),
                            status: "typing".to_string(),
                        };
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    &event,
                                    json!({"is_typing": is_typing}),
                                )
                                .with_redis_presence(presence),
                            )
                            .await;
                    }
                }
            } else {
                for member in members {
                    let _ = state
                        .dispatch(AppEvent::user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({"conversation_id": conversation_id,"is_typing": is_typing,"user_id": "nomi","text":""}),
                        ))
                        .await;
                }

                if !is_group {
                    info!("send_status_update channel:{:?}", ch_names);
                    let channel_info = sqlx::query!(
                        "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                        conversation_id,
                        &ch_names[..]
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or(Vec::new());

                    for channel in channel_info {
                        let presence = PresenceMessage {
                            sender_id: channel.external_group_id.clone(),
                            chat_id: channel.external_group_id.clone(),
                            channel: channel.channel.clone(),
                            status: "typing".to_string(),
                        };
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    &event,
                                    json!({"is_typing": is_typing}),
                                )
                                .with_redis_presence(presence),
                            )
                            .await;
                    }
                }
            }
        }
    });
}

pub async fn send_message_to_subscriber(
    state: &AppState,
    members: Vec<Uuid>,
    conversation_id: Uuid,
    source: MessageSource,
    sse_data: serde_json::Value,
    data: MessageItem,
) {
    let state = state.clone();
    let pool = state.pool.clone();
    let outbound_message = data.clone();
    tokio::spawn(async move {
        let convo = sqlx::query!(
            "SELECT conversation_type,id FROM conversations WHERE id = $1",
            conversation_id
        )
        .fetch_one(&pool)
        .await;

        let ch_names = match source.clone() {
            MessageSource::Web { name } => vec![name.to_string()],
            MessageSource::Telegram { name } => vec![name.to_string()],
            MessageSource::WhatsApp { name } => vec![name.to_string()],
            MessageSource::Other { name } => vec![name.to_string()],
            MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(convo) = convo {
            for member in members {
                let _ = state
                    .dispatch(AppEvent::user(
                        member.to_string().as_str(),
                        "message",
                        sse_data.clone(),
                    ))
                    .await;
            }

            // --- Multi-bubble Sequential Burst Strategy ---
            let bubbles = crate::common::splitter::split_into_bubbles(&outbound_message.content);

            if convo.conversation_type.eq_ignore_ascii_case("private") {
                let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id, cm.user_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or(Vec::new());

                for channel in channel_info {
                    for (i, bubble_text) in bubbles.iter().enumerate() {
                        let outbound = OutboundMessage {
                            is_group: false,
                            sender_id: channel.external_id.clone(),
                            conversation_id: channel.external_chat_id.clone(),
                            text: bubble_text.clone(),
                            channel: channel.channel_type.clone(),
                            // Attach media only to the first bubble
                            video_url: if i == 0 {
                                outbound_message.video_url.clone()
                            } else {
                                None
                            },
                            image_url: if i == 0 {
                                outbound_message.image_url.clone()
                            } else {
                                None
                            },
                            audio_url: if i == 0 {
                                outbound_message.audio_url.clone()
                            } else {
                                None
                            },
                            doc_url: if i == 0 {
                                outbound_message.document_url.clone()
                            } else {
                                None
                            },
                            sticker_url: if i == 0 {
                                outbound_message.sticker_url.clone()
                            } else {
                                None
                            },
                            metadata: None,
                        };
                        info!("Sent outbound message: {}", outbound);
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    "message",
                                    json!({"text": bubble_text}),
                                )
                                .with_redis_outbound(outbound),
                            )
                            .await;

                        if i < bubbles.len() - 1 {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        }
                    }
                }
            } else {
                let channel_info = sqlx::query!(
                    "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                    conversation_id,
                    &ch_names[..]
                )
                .fetch_all(&pool)
                .await
                .unwrap_or(Vec::new());

                for channel in channel_info {
                    for (i, bubble_text) in bubbles.iter().enumerate() {
                        let outbound = OutboundMessage {
                            is_group: false,
                            sender_id: "".to_string(),
                            conversation_id: channel.external_group_id.clone(),
                            text: bubble_text.clone(),
                            channel: channel.channel.clone(),
                            // Attach media only to the first bubble
                            video_url: if i == 0 {
                                outbound_message.video_url.clone()
                            } else {
                                None
                            },
                            image_url: if i == 0 {
                                outbound_message.image_url.clone()
                            } else {
                                None
                            },
                            audio_url: if i == 0 {
                                outbound_message.audio_url.clone()
                            } else {
                                None
                            },
                            doc_url: if i == 0 {
                                outbound_message.document_url.clone()
                            } else {
                                None
                            },
                            sticker_url: if i == 0 {
                                outbound_message.sticker_url.clone()
                            } else {
                                None
                            },
                            metadata: None,
                        };
                        info!("Sent outbound message: {}", outbound);
                        let _ = state
                            .dispatch(
                                AppEvent::conversation(
                                    conversation_id,
                                    "message",
                                    json!({"text": bubble_text}),
                                )
                                .with_redis_outbound(outbound),
                            )
                            .await;

                        if i < bubbles.len() - 1 {
                            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        }
                    }
                }
            }
        }
    });
}
