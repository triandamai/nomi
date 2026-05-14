use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::model::{MessageSource, UnifiedMessage};
use crate::feature::{OutboundMessage, PresenceMessage};
use crate::rag;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::common::agent::classification::{classification, fetch_media_from_storage};
use crate::common::repository::message_repo::save_message;
use crate::feature::conversation::model::MessageItem;
use crate::rag::trigger_memory_consolidation;
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
        let pool = state.pool.clone();
        let m_url = media_url.to_string();
        let m_type = media_type.to_string();
        tokio::spawn(async move {
            let _ = crate::common::repository::pending_media_repo::upsert_pending_media(
                &pool,
                conversation_id,
                &m_url,
                &m_type,
                None,
            )
            .await;
        });

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
        // Task 1: If message.text is NOT empty: Do not ask for clarification.
        // Combine the text and the image into a single multi-part prompt for Gemini.
        let injected_system_prompt =
            crate::prompts::PromptRegistry::media_with_text_instruction().to_string();
        process_v2_message_with_intent(
            state.clone(),
            msg,
            text_content,
            Some(injected_system_prompt),
        )
        .await
    } else {
        process_v2_message_with_intent(state.clone(), msg, text_content, None).await
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
    let save_user_message = save_message(
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
    .await;
    if let Err(e) = save_user_message {
        info!("Saving message failed :{}", e);
        return Ok(());
    }

    let members = sqlx::query!(
        "SELECT m.user_id FROM conversation_members as m WHERE m.conversation_id = $1",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or(Vec::new());

    let saved_message = save_user_message?;

    for member in members.iter().map(|v|v.user_id.clone()) {
        let _ = state.send_sse_to_user(
            member.to_string().as_str(),
            "message",
            json!({
                    "id": saved_message.id,
                    "conversation_id":conversation_id,
                    "role": saved_message.role,
                    "content": saved_message.content.clone(),
                    "thought": saved_message.thought,
                    "user_id": saved_message.user_id,
                    "total_tokens": 0,
                    "image_url": saved_message.image_url.as_ref().map(|path| state.storage.get_full_url(path).to_string()),
                    "created_at": saved_message.created_at
        })).await;
    }
    // Group is registered, only respond if mentioned
    if msg.is_group
        && !msg.is_mentioned
        && (msg.image_url.is_none()
            && msg.video_url.is_none()
            && msg.audio_url.is_none()
            && msg.doc_url.is_none()
            && msg.sticker_url.is_none())
    {
        info!(
            "Message is from registered group, but not mentioned or image, ignoring. but immediate save for beter context history"
        );

        return Ok(());
    }

    // Fetch updated total tokens and broadcast
    if let Ok(row) = sqlx::query!(
        "SELECT cumulative_tokens FROM conversations WHERE id = $1",
        conversation_id
    )
    .fetch_one(&state.pool)
    .await
    {
        let _ = state
            .broadcast_sse(
                "token_update",
                json!({
                    "conversation_id": conversation_id,
                    "cumulative_tokens": row.cumulative_tokens
                }),
            )
            .await;
    }

    if text_content.trim().eq_ignore_ascii_case("skip") {
        info!("Skip instruction received, marking last media as processed");
        let _ = crate::common::repository::message_repo::mark_last_media_processed(
            &state.pool,
            conversation_id,
        )
        .await;
    }

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
        members.iter().map(|v| v.user_id.clone()).collect(),
        conversation_id,
        &msg,
        text_content.clone(),
        injected_system_prompt,
    )
    .await;

    // Fetch media data if present for Multi-Part prompt
    let media_data = if let Some(ref url) = msg.image_url {
        fetch_media_from_storage(&state, url).await.ok()
    } else {
        None
    };

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

        let timezone_str = "Asia/Jakarta"; // Default to Trian's timezone
        let tz: chrono_tz::Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);
        let now_local = Utc::now().with_timezone(&tz);

        combined.push_str(&format!(
            "\n### Current Contextual Time\n- UTC: {}\n- Local Time: {} ({})\n",
            Utc::now().to_rfc3339(),
            now_local.to_rfc3339(),
            timezone_str
        ));

        combined.push_str("\n### Timezone Instructions\n");
        combined.push_str(&format!(
            "The user's current local time is {} ({}). When the user asks for a time like \"6:00\", assume they mean this local time and calculate the UTC equivalent for storage using the +07:00 offset or by converting from Asia/Jakarta. ALWAYS provide the due_at in ISO 8601 format including the local offset (e.g., {} ) for the tool call, but you can acknowledge the local time in your thoughts.\n",
            now_local.format("%H:%M"),
            timezone_str,
            now_local.format("%Y-%m-%dT%H:%M:%S%z")
        ));

        combined.push_str("\n### Orchestrator Instructions \n");
        combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());

        if msg.is_mentioned && augmented_text.len() < 10 {
            combined.push_str("\n[SYSTEM: The user has activated you with a short mention. Look back at the last 3-5 messages in the 'Recent History' to find the intent (URL, Image, Question) and act on it immediately.]\n");
        }

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
                .or_else(|| {
                    meta.get("is_processed")
                        .and_then(|v| v.as_str().map(|s| s == "true"))
                })
                .unwrap_or(false)
        } else {
            false
        };

        let image_url = match msg.image_url {
            Some(path) if !is_processed => {
                format!(" - Image: {} \n", state.storage.get_full_url(&path))
            }
            _ => "".to_string(),
        };
        let video_url = match msg.video_url {
            Some(path) if !is_processed => {
                format!("- Video: {} \n", state.storage.get_full_url(&path))
            }
            _ => "".to_string(),
        };
        let audio_url = match msg.audio_url {
            Some(path) if !is_processed => {
                format!(" - Audio: {} \n", state.storage.get_full_url(&path))
            }
            _ => "".to_string(),
        };
        let document_url = match msg.document_url {
            Some(path) if !is_processed => {
                format!("- Document: {} \n", state.storage.get_full_url(&path))
            }
            _ => "".to_string(),
        };

        let sticker_url = match msg.sticker_url {
            Some(path) if !is_processed => {
                format!("- Sticker: {} \n", state.storage.get_full_url(&path))
            }
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
    let memories_text = if embedding.is_ok() {
        crate::utils::rag::hybrid_retrieve(
            &state.pool,
            &augmented_text,
            embedding.unwrap().embedding.values,
            Some(conversation_id),
            None,
            None,
        )
        .await
        .unwrap_or_default()
        .join("---")
    } else {
        String::new()
    };

    // --- V2 Autonomous Loop ---
    let mut loop_count = 0;
    let max_loops = 10;

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
            media: media_data.clone(),
        };

        // Status: Model is thinking
        if loop_count <= 1 {
            send_status_update(
                &state,
                members.iter().map(|v| v.user_id).collect(),
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
                    turn_text.push_str("");

                    accumulated_thought.push_str(&chunk.thought);
                    accumulated_thought.push_str("");

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
                    accumulated_content.push_str("");
                }

                // Append model's output to history_text to ensure context persists across the loop turns
                if !turn_text.is_empty() {
                    history_text.push_str(&format!(
                        "-[{}] Nomi: {}.",
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
                    && (finish_reason.eq_ignore_ascii_case("stop") || finish_reason.is_empty())
                {
                    // Synthesis Turn check: If we have tool results but haven't written a conversational response yet,
                    // we might need to force the model to synthesize.
                    let content_is_empty =
                        strip_thinking_tags(&accumulated_content).trim().is_empty();
                    if !tool_turns.is_empty() && content_is_empty && loop_count < max_loops {
                        info!(
                            "Synthesis Turn: Model tried to stop after tools without content. Forcing synthesis turn."
                        );
                        // Force history update to reflect tool results were seen
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
                        members.iter().map(|v| v.user_id).collect(),
                        conversation_id,
                        msg.source.clone(),
                        "tool_start".to_string(),
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
                        "-[{}] System (Tool {} Result): {}. [STATUS: {}]",
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
                members.iter().map(|v| v.user_id).collect(),
                conversation_id,
                msg.source.clone(),
                payload,
                record.clone(),
            );

            // Fetch updated total tokens and broadcast
            if let Ok(row) = sqlx::query!(
                "SELECT cumulative_tokens FROM conversations WHERE id = $1",
                conversation_id
            )
            .fetch_one(&state.pool)
            .await
            {
                let _ = state
                    .broadcast_sse(
                        "token_update",
                        json!({
                            "conversation_id": conversation_id,
                            "cumulative_tokens": row.cumulative_tokens
                        }),
                    )
                    .await;
            }
        }

        let pool = state.pool.clone();
        let gemini = state.gemini.clone();
        let gemini_api_key = state.gemini_api_key.clone();
        let sse = state.sse.clone();
        tokio::spawn(async move {
            let _ =
                trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id, sse)
                    .await;
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
    let healed = crate::common::format::heal_thinking_tags(text);
    let re = regex::Regex::new(r"(?s)<thinking>.*?</thinking>|<thinking>.*").unwrap();
    re.replace_all(&healed, "").trim().to_string()
}

pub fn send_status_update(
    state: &AppState,
    members: Vec<Uuid>,
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
            if data.conversation_type.eq_ignore_ascii_case("private") {
                info!("send_status_update web");
                for member in members {
                    let _ = state
                        .send_sse_to_user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "text":text
                            }),
                        )
                        .await;
                }
                match source {
                    MessageSource::Web { .. } => {
                        //keep update we event thought the channel isnt from we
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
                for member in members {
                    let _ = state
                        .send_sse_to_user(
                            member.to_string().as_str(),
                            event.to_string().as_str(),
                            json!({
                                "conversation_id": conversation_id,
                                "text":text
                            }),
                        )
                        .await;
                }
                match source {
                    MessageSource::Web { .. } => {
                        //keep update we event thought the channel isnt from we
                        info!("send_status_update web");
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
            for member in members {
                let _ = state
                    .send_sse_to_user(member.to_string().as_str(), "message", sse_data.clone())
                    .await;
            }

            // --- Multi-bubble Sequential Burst Strategy ---
            let bubbles = crate::common::splitter::split_into_bubbles(&outbound_message.content);

            if data.conversation_type.eq_ignore_ascii_case("private") {
                match source {
                    MessageSource::Web { .. } => {
                        //since we always send sse to web no matter channel is come from
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
                                let _ = state.publish_outbond(&outbound).await;

                                if i < bubbles.len() - 1 {
                                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                }
                            }
                        }
                    }
                }
            } else {
                match source {
                    MessageSource::Web { .. } => {
                        //since we always send sse to web no matter channel is come from
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
                                let _ = state.publish_outbond(&outbound).await;

                                if i < bubbles.len() - 1 {
                                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
