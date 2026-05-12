use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::{OutboundMessage, PresenceMessage};
use crate::rag;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::common::agent::classification::classification;
use crate::common::repository::message_repo::save_message;
use crate::feature::conversation::chat_model::MessageItem;
use crate::feature::message_processor::MessageSource;
use crate::feature::message_processor::processor::{
     trigger_memory_consolidation,
};
use tracing::{error, info};


pub async fn process_v2_message(state: AppState, msg: UnifiedMessage) -> anyhow::Result<()> {
    let conversation_id = msg.conversation_id;
    let text_content = msg.text_content.clone();

    info!(
        conversation_id = %conversation_id,
        user_id = ?msg.user_id,
        source = ?msg.source,
        "Processing unified message v2"
    );

    // 0. Zero-Intent Guard: Media with EMPTY text
    let has_media = msg.image_url.is_some()
        || msg.video_url.is_some()
        || msg.audio_url.is_some()
        || msg.doc_url.is_some()
        || msg.sticker_url.is_some();

    let is_skip = text_content.trim().eq_ignore_ascii_case("skip");

    if has_media && text_content.trim().is_empty() {
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

        info!("Zero-intent {} detected: {}", media_type, media_url);

        // Save to pending_media table for Media Checkpoint System
        let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
            &state.pool,
            conversation_id,
            media_url,
            media_type,
            None,
        )
            .await;

        // Instead of hardcoded clarification, we inject a system prompt to the LLM to ask for clarification.
        // This will be passed to process_v2_message_with_intent but NOT saved as a message.
        let injected_system_prompt =
            crate::prompts::PromptRegistry::zero_intent_clarification().to_string();

        process_v2_message_with_intent(
            state.clone(),
            msg,
            format!("[User uploaded a {}]", media_type),
            Some(injected_system_prompt),
        )
            .await
    } else if has_media && !is_skip {
        // Instead of hardcoded clarification, we inject a system prompt to the LLM to ask for clarification.
        // This will be passed to process_v2_message_with_intent but NOT saved as a message.
        let injected_system_prompt =
            crate::prompts::PromptRegistry::media_intent_clarification().to_string();
        process_v2_message_with_intent(
            state.clone(),
            msg,
            text_content,
            Some(injected_system_prompt),
        )
            .await
    } else {
        process_v2_message_with_intent(
            state.clone(),
            msg,
            text_content,
            None,
        )
            .await
    }
}

async fn process_v2_message_with_intent(
    state: AppState,
    msg: UnifiedMessage,
    text_content: String,
    injected_system_prompt: Option<String>,
) -> anyhow::Result<()> {
    let conversation_id = msg.conversation_id;
    let user_id = msg.user_id;

    info!(
        conversation_id = %conversation_id,
        user_id = ?user_id,
        "Processing v2 message loop with intent"
    );

    // 1. Immediate Save (Only the actual user content)
    let m = save_message(
        &state.pool,
        conversation_id,
        "user",
        &text_content,
        None,
        user_id,
        0,
        0,
        0,
        msg.image_url.clone(),
        msg.video_url.clone(),
        msg.audio_url.clone(),
        msg.doc_url.clone(),
        None,
    )
        .await?;

    if text_content.trim().eq_ignore_ascii_case("skip") {
        info!("Skip instruction received, marking last media as processed");
        let _ = crate::common::repository::message_repo::mark_last_media_processed(&state.pool, conversation_id).await;
    }

    let payload = json!({
        "id": m.id,
        "conversation_id": conversation_id,
        "role": m.role,
        "content": m.content,
        "thought": m.thought,
        "user_id": m.user_id,
        "image_url": m.image_url.as_ref().map(|path| state.storage.get_full_url(path)),
        "created_at": m.created_at,
        "total_tokens": 0,
    });
    let _ = match user_id {
        None => state.broadcast_sse("message", payload).await,
        Some(ref id) => {
            state
                .send_sse_to_user(id.to_string().as_str(), "message", payload)
                .await
        }
    };

    let presence_payload = json!({
        "conversation_id": conversation_id,
        "is_typing": true,
        "user_id": "nomi"
    });
    let _ = match user_id {
        None => state.broadcast_presence_sse(presence_payload).await,
        Some(ref id) => {
            state
                .send_presence_sse_to_user(id.to_string().as_str(), presence_payload)
                .await
        }
    };

    if let Ok(channel_info) = sqlx::query!(
        "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
        conversation_id
    ).fetch_all(&state.pool).await {
        for channel in channel_info {
            let presence = PresenceMessage {
                sender_id: channel.external_id.clone(),
                chat_id: channel.external_chat_id.clone(),
                channel: channel.channel_type.clone(),
                status: "typing".to_string(),
            };
            let _ = state.publish_presence(&presence).await;
        }
    }

    let (augmented_text, _media_context) = classification(
        &state,
        conversation_id,
        &msg,
        text_content.clone(),
        injected_system_prompt,
    )
        .await;

    let dispatcher = ToolDispatcher::new(
        state.pool.clone(),
        std::env::current_dir().unwrap_or_default(),
        user_id.clone(),
        Some(conversation_id),
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.sse.clone(),
        state.storage.clone(),
    );

    let conversation = sqlx::query!(
        "SELECT bootstrap_content, soul_content, metadata FROM conversations WHERE id = $1",
        conversation_id
    )
        .fetch_one(&state.pool)
        .await?;

    let system_prompt = {
        let boot = conversation.bootstrap_content.unwrap_or_default();
        let soul = conversation.soul_content.unwrap_or_default();
        let mut combined = boot;
        if !soul.is_empty() {
            combined.push_str("\n### Current Personality/Soul\n");
            combined.push_str(&soul);
        }

        combined.push_str("\n### Orchestrator Instructions \n");
        combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());
        combined.push_str("");
        combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());
        combined
    };

    let history = sqlx::query!(
        "SELECT
                users.display_name as display_name,
                messages.created_at,
                messages.role,
                messages.content,
                messages.image_url,
                messages.video_url,
                messages.audio_url,
                messages.document_url,
                messages.sticker_url,
                messages.metadata
            FROM messages LEFT JOIN users ON users.id = messages.user_id
            WHERE conversation_id = $1
            ORDER BY created_at
        DESC LIMIT 15",
        conversation_id
    )
        .fetch_all(&state.pool)
        .await?;

    let mut history_text = String::new();
    for msg in history.into_iter().rev() {
        let is_processed = if let Some(meta) = msg.metadata {
            meta.get("is_processed")
                .and_then(|v| v.as_bool())
                .or_else(|| meta.get("is_processed").and_then(|v| v.as_str().map(|s| s == "true")))
                .unwrap_or(false)
        } else {
            false
        };

        let image_url = match msg.image_url {
            Some(path) if !is_processed => format!(" - Image: {} \n", state.storage.get_full_url(&path)),
            _ => "".to_string(),
        };
        let video_url = match msg.video_url {
            Some(path) if !is_processed => format!("- Video: {} \n", state.storage.get_full_url(&path)),
            _ => "".to_string(),
        };
        let audio_url = match msg.audio_url {
            Some(path) if !is_processed => format!(" - Audio: {} \n", state.storage.get_full_url(&path)),
            _ => "".to_string(),
        };
        let document_url = match msg.document_url {
            Some(path) if !is_processed => format!("- Document: {} \n", state.storage.get_full_url(&path)),
            _ => "".to_string(),
        };

        let sticker_url = match msg.sticker_url {
            Some(path) if !is_processed => format!("- Sticker: {} \n", state.storage.get_full_url(&path)),
            _ => "".to_string(),
        };
        let role_label = match msg.role.as_str() {
            "user" => match msg.display_name {
                None => "User".to_string(),
                Some(ref user) => user.clone(),
            },
            "assistant" => "Nomi".to_string(),
            _ => "System".to_string(),
        };
        history_text.push_str(&format!(
            "-[{}] {}: {}.\n {}{}{}{}{}",
            msg.created_at
                .unwrap_or(Utc::now())
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            role_label,
            msg.content,
            image_url,
            video_url,
            audio_url,
            document_url,
            sticker_url
        ));
    }

    let embedding = rag::get_embedding(&state.gemini_api_key, &augmented_text).await;
    let memories_text = if !embedding.is_ok() {
        crate::utils::rag::hybrid_retrieve(
            &state.pool,
            &augmented_text,
            embedding.unwrap().embedding.values,
            Some(conversation_id),
        )
            .await
            .unwrap_or_default()
            .join("---")
    } else {
        String::new()
    };

    // --- V2 Autonomous Loop ---
    let mut loop_count = 0;
    let max_loops = 5;

    let mut final_response = None;
    let mut tool_turns = Vec::new();

    let mut accumulated_content = String::new();
    let mut accumulated_thought = String::new();
    let mut total_prompt_tokens = 0;
    let mut total_answer_tokens = 0;
    let mut total_tokens = 0;

    while loop_count < max_loops {
        loop_count += 1;
        info!("V2 Loop iterate(N): N({})", loop_count);

        let current_actor = PromptActor::MultiTool {
            history: history_text.clone(),
            memories: memories_text.clone(),
            message: augmented_text.clone(),
            system_prompt: system_prompt.clone(),
            tool_turns: tool_turns.clone(),
        };

        // Status: Model is thinking
        if loop_count <= 1 {
            send_status_update(
                &state,
                conversation_id,
                msg.source.clone(),
                "thought".to_string(),
                crate::prompts::StatusRegistry::random_thinking_phrase(),
            );
        }

        let result = crate::common::agent::send_prompt(state.gemini.as_ref(), current_actor).await;

        match result {
            Ok((response, chunk)) => {
                let mut turn_text = String::new();
                if !chunk.thought.is_empty() {
                    turn_text.push_str(&chunk.thought);
                    turn_text.push_str(
                        "
",
                    );

                    accumulated_thought.push_str(&chunk.thought);
                    accumulated_thought.push_str(
                        "
",
                    );

                    let payload =
                        json!({ "thought": chunk.thought, "conversation_id": conversation_id });
                    let _ = match user_id {
                        None => state.broadcast_sse("thought", payload).await,
                        Some(ref id) => {
                            state
                                .send_sse_to_user(id.to_string().as_str(), "thought", payload)
                                .await
                        }
                    };
                }
                if !chunk.content.is_empty() {
                    turn_text.push_str(&chunk.content);

                    accumulated_content.push_str(&chunk.content);
                    accumulated_content.push_str(
                        "

",
                    );
                }

                // Append model's output to history_text to ensure context persists across the loop turns
                if !turn_text.is_empty() {
                    history_text.push_str(&format!(
                        "-[{}] Nomi: {}.
",
                        Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                        turn_text
                    ));
                }

                total_prompt_tokens += chunk.prompt_tokens;
                total_answer_tokens += chunk.answer_tokens;
                total_tokens += chunk.total_tokens;

                let tool_calls = response.function_calls();
                let finish_reason = chunk.finish_reason.clone().unwrap_or_default();

                if tool_calls.is_empty()
                    && (finish_reason.contains("Stop") || finish_reason.is_empty())
                {
                    // Synthesis Turn check: If we have tool results but haven't written a conversational response yet,
                    // we might need to force the model to synthesize.
                    let content_is_empty =
                        strip_thinking_tags(&accumulated_content).trim().is_empty();
                    if !tool_turns.is_empty() && content_is_empty && loop_count < max_loops {
                        info!(
                            "Synthesis Turn: Model tried to stop after tools without content. Forcing synthesis turn."
                        );
                        continue;
                    }

                    let mut final_chunk = chunk.clone();
                    final_chunk.content =
                        strip_thinking_tags(&accumulated_content).trim().to_string();
                    final_chunk.thought = accumulated_thought.trim().to_string();
                    final_chunk.prompt_tokens = total_prompt_tokens;
                    final_chunk.answer_tokens = total_answer_tokens;
                    final_chunk.total_tokens = total_tokens;

                    final_response = Some((response, final_chunk));
                    break;
                }

                if loop_count >= max_loops {
                    let mut final_chunk = chunk.clone();
                    final_chunk.content =
                        strip_thinking_tags(&accumulated_content).trim().to_string();
                    final_chunk.thought = accumulated_thought.trim().to_string();
                    final_chunk.prompt_tokens = total_prompt_tokens;
                    final_chunk.answer_tokens = total_answer_tokens;
                    final_chunk.total_tokens = total_tokens;

                    final_response = Some((response, final_chunk));
                    break;
                }

                let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();

                // Status: Tool checking
                for call in &current_calls {
                    send_status_update(
                        &state,
                        conversation_id,
                        msg.source.clone(),
                        "thought".to_string(),
                        crate::prompts::StatusRegistry::random_action_phrase(&call.name),
                    );
                }

                let tool_results = execute_tools(
                    &dispatcher,
                    current_calls.clone(),
                    &text_content, // use the v2-stripped one
                    Some(state.sse.clone()),
                )
                    .await;

                // Append Tool Responses to history_text to enforce memory management persistence
                for (name, result) in &tool_results {
                    history_text.push_str(&format!(
                        "-[{}] System (Tool {} Result): {}. [STATUS: {}]
",
                        Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                        name,
                        if result.success {
                            &result.content
                        } else {
                            &result.error
                        },
                        if result.success { "SUCCESS" } else { "ERROR" }
                    ));
                }

                tool_turns.push((current_calls, tool_results));
            }
            Err(e) => {
                error!("V2 Agentic loop error: {}", e);
                break;
            }
        }
    }

    if let Some((_, function_result)) = final_response {
        // Double-check thinking tags are stripped (they already are above, but for clarity)
        let sanitized_content = strip_thinking_tags(&function_result.content)
            .trim()
            .to_string();

        if let Ok(record) = save_message(
            &state.pool,
            conversation_id,
            "assistant",
            &sanitized_content,
            Some(function_result.thought.as_str()),
            None,
            function_result.prompt_tokens,
            function_result.answer_tokens,
            function_result.total_tokens,
            None,
            None,
            None,
            None,
            None,
        )
            .await
        {
            let payload = json!({
                        "id": record.id,
                        "conversation_id":conversation_id,
                        "role": record.role,
                        "content": record.content.clone(),
                        "thought": record.thought,
                        "user_id": record.user_id,
                        "total_tokens": function_result.total_tokens,
                        "image_url": record.image_url.as_ref().map(|path| state.storage.get_full_url( path)),
                        "created_at": record.created_at
            });

            send_message_to_subscriber(
                &state,
                conversation_id,
                msg.source.clone(),
                payload,
                record.clone(),
            );
        }

        let pool = state.pool.clone();
        let gemini = state.gemini.clone();
        let gemini_api_key = state.gemini_api_key.clone();
        tokio::spawn(async move {
            let _ =
                trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id).await;
        });

        let payload = json!({
            "conversation_id": conversation_id,
            "is_typing": false,
            "user_id": "nomi"
        });

        let _ = match user_id {
            None => state.broadcast_presence_sse(payload).await,
            Some(ref id) => {
                state
                    .send_presence_sse_to_user(id.to_string().as_str(), payload)
                    .await
            }
        };

        if let Ok(channel_info) = sqlx::query!(
        "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
            conversation_id
        ).fetch_all(&state.pool).await {

            for channel in channel_info {
                let presence = PresenceMessage {
                    sender_id: channel.external_id.clone(),
                    chat_id: channel.external_chat_id.clone(),
                    channel: channel.channel_type.clone(),
                    status: "idle".to_string(),
                };
                let _ = state.publish_presence(&presence).await;

            }
        }
        return Ok(());
    }
    Ok(())
}

fn strip_thinking_tags(text: &str) -> String {
    let re = regex::Regex::new(r"(?s)<thinking>.*?</thinking>|<thinking>.*").unwrap();
    let stripped = re.replace_all(text, "").trim().to_string();

    // Refined logic: If the message starts with "thinking" (case insensitive) but lacked tags,
    // attempt to strip the first paragraph which is likely leaked monologue.
    if stripped.to_lowercase().starts_with("thinking") {
        let paragraphs: Vec<&str> = stripped.split("\n\n").collect();
        if paragraphs.len() > 1 {
            return paragraphs[1..].join("\n\n").trim().to_string();
        }
    }
    stripped
}

pub fn send_status_update(
    state: &AppState,
    conversation_id: Uuid,
    source: MessageSource,
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

        let ch_name = match source.clone() {
            MessageSource::Web { name } => name,
            MessageSource::Telegram { name } => name,
            MessageSource::WhatsApp { name } => name,
            MessageSource::Other { name } => name,
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(data) = convo {
            let members = sqlx::query!(
                "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
                data.id
            )
            .fetch_all(&pool)
            .await
            .unwrap_or(Vec::new());

            if data.conversation_type.eq_ignore_ascii_case("private") {
                match source {
                    MessageSource::Web { .. } => {
                        info!("send_status_update web");
                        for member in members {
                            let _ = state
                                .send_sse_to_user(
                                    member.user_id.to_string().as_str(),
                                    event.to_string().as_str(),
                                    json!({}),
                                )
                                .await;
                        }
                    }
                    MessageSource::Other { name } => {
                        info!(
                            "Sent status update failed, channel not supported : {}",
                            name
                        );
                    }
                    _ => {
                        info!("send_status_update channel:{}", ch_name);
                        let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = $2",
                            conversation_id,
                            ch_name
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let outbound = OutboundMessage {
                                is_group: false,
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
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            } else {
                match source {
                    MessageSource::Web { .. } => {
                        info!("send_status_update web");
                        for member in members {
                            let _ = state
                                .send_sse_to_user(
                                    member.user_id.to_string().as_str(),
                                    event.to_string().as_str(),
                                    json!({}),
                                )
                                .await;
                        }
                    }
                    MessageSource::Other { name } => {
                        info!(
                            "Sent status update failed, channel not supported : {}",
                            name
                        );
                    }
                    _ => {
                        info!("send_status_update channel:{}", ch_name);
                        let channel_info = sqlx::query!(
                            "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel = $2",
                            conversation_id,
                            ch_name
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
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            }
        }
    });
}

pub fn send_message_to_subscriber(
    state: &AppState,
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

        let ch_name = match source.clone() {
            MessageSource::Web { name } => name,
            MessageSource::Telegram { name } => name,
            MessageSource::WhatsApp { name } => name,
            MessageSource::Other { name } => name,
        };

        if let Err(err) = &convo {
            info!("Sent status update failed: {}", err);
        }

        if let Ok(data) = convo {
            let members = sqlx::query!(
                "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
                data.id
            )
            .fetch_all(&pool)
            .await
            .unwrap_or(Vec::new());

            if data.conversation_type.eq_ignore_ascii_case("private") {
                match source {
                    MessageSource::Web { .. } => {
                        for member in members {
                            let _ = state
                                .send_sse_to_user(
                                    member.user_id.to_string().as_str(),
                                    "message",
                                    sse_data.clone(),
                                )
                                .await;
                        }
                    }
                    MessageSource::Other { name } => {
                        info!(
                            "Sent status update failed, channel not supported : {}",
                            name
                        );
                    }
                    _ => {
                        let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = $2",
                            conversation_id,
                            ch_name
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: channel.external_id.clone(),
                                conversation_id: channel.external_chat_id.clone(),
                                text: outbound_message.content.clone(),
                                channel: channel.channel_type.clone(),
                                video_url: outbound_message.video_url.clone(),
                                image_url: outbound_message.image_url.clone(),
                                audio_url: outbound_message.audio_url.clone(),
                                doc_url: outbound_message.document_url.clone(),
                                sticker_url: outbound_message.sticker_url.clone(),
                                metadata: None,
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            } else {
                match source {
                    MessageSource::Web { .. } => {
                        for member in members {
                            let _ = state
                                .send_sse_to_user(
                                    member.user_id.to_string().as_str(),
                                    "message",
                                    sse_data.clone(),
                                )
                                .await;
                        }
                    }
                    MessageSource::Other { name } => {
                        info!(
                            "Sent status update failed, channel not supported : {}",
                            name
                        );
                    }
                    _ => {
                        let channel_info = sqlx::query!(
                            "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel = $2",
                            conversation_id,
                            ch_name
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: "".to_string(),
                                conversation_id: channel.external_group_id.clone(),
                                text: outbound_message.content.clone(),
                                channel: channel.channel,
                                video_url: outbound_message.video_url.clone(),
                                image_url: outbound_message.image_url.clone(),
                                audio_url: outbound_message.audio_url.clone(),
                                doc_url: outbound_message.document_url.clone(),
                                sticker_url: outbound_message.sticker_url.clone(),
                                metadata: None,
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            }
        }
    });
}