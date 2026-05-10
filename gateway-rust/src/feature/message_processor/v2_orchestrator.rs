use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::media::{MediaClassification};
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::{OutboundMessage, PresenceMessage};
use crate::rag;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::common::repository::message_repo::save_message;
use tracing::{error, info};
use crate::feature::message_processor::processor::{
    classify_media_context, extract_expense_data, extract_maintenance_data,
    extract_technical_doc, trigger_memory_consolidation,
};

fn strip_thinking_tags(text: &str) -> String {
    let re = regex::Regex::new(r"(?s)<thinking>.*?(?:</thinking>|$)").unwrap();
    re.replace_all(text, "").trim().to_string()
}

fn send_status_update(pool: &sqlx::PgPool, conversation_id: Uuid, text: String) {
    let pool = pool.clone();
    tokio::spawn(async move {
        let channel_info = sqlx::query!(
            "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
            conversation_id
        ).fetch_all(&pool).await.unwrap_or_default();

        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            if let Ok(client) = redis::Client::open(redis_url) {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    use redis::AsyncCommands;
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
                        let payload = serde_json::to_string(&outbound).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:outbound", payload).await;
                    }
                }
            }
        }
    });
}

pub async fn process_v2_message(state: AppState, msg: UnifiedMessage) -> anyhow::Result<()> {
    let conversation_id = msg.conversation_id;
    let user_id = msg.user_id;
    let text_content = msg.text_content.clone(); // Keep original text without v2 prefix?
    
    // We should strip "v2 " from the beginning if it exists, otherwise use as is.
    let text_content = if text_content.starts_with("v2 ") {
        text_content.replacen("v2 ", "", 1)
    } else {
        text_content
    };

    info!(
        conversation_id = %conversation_id,
        user_id = ?user_id,
        source = ?msg.source,
        "Processing unified message v2"
    );

    // 1. Immediate Save
    let m = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id, created_at) VALUES ($1, 'user', $2, '', $3, now()) RETURNING id, role, content, thought, created_at, user_id",
        conversation_id,
        text_content,
        user_id
    ).fetch_one(&state.pool).await?;

    let payload = json!({
        "id": m.id,
        "conversation_id": conversation_id,
        "role": m.role,
        "content": m.content,
        "thought": m.thought,
        "user_id": m.user_id,
        "created_at": m.created_at.unwrap_or_else(Utc::now),
        "total_tokens": 0,
    });
    let _ = match user_id {
        None => state.broadcast_sse("message", payload).await,
        Some(ref id) => state.send_sse_to_user(id.to_string().as_str(), "message", payload).await
    };

    let presence_payload = json!({
        "conversation_id": conversation_id,
        "is_typing": true,
        "user_id": "nomi"
    });
    let _ = match user_id {
        None => state.broadcast_presence_sse(presence_payload).await,
        Some(ref id) => state.send_presence_sse_to_user(id.to_string().as_str(), presence_payload).await
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
            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        use redis::AsyncCommands;
                        let payload = serde_json::to_string(&presence).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:presence", payload).await;
                    }
                }
            }
        }
    }

    let mut media_context = String::new();
    if let Some(image_url) = msg.image_url {
        info!("Media detected, classifying: {}", image_url);
        let classification = classify_media_context(&state, &image_url).await.unwrap_or(MediaClassification::Other);
        match classification {
            MediaClassification::ExpenseReceipt => {
                if let Ok(expense) = extract_expense_data(&state, &image_url).await {
                    media_context = format!(
                        "
[SYSTEM: User uploaded an expense receipt. Merchant: {}, Total: {}, Category: {}. Items: {}]",
                        expense.merchant, expense.total, expense.category, expense.items.join(", ")
                    );
                    let memory_content = format!("Expense at {}: {} ({})", expense.merchant, expense.total, expense.category);
                    if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &memory_content).await {
                        let metadata = json!({
                            "type": "memory",
                            "source": "image_classification",
                            "classification": "EXPENSE_RECEIPT",
                            "data": expense,
                            "image_url": image_url
                        });
                        let _ = rag::save_to_knowledge_base(&state.pool, &memory_content, embedding, Some(metadata), Some(conversation_id)).await;
                    }
                }
            }
            MediaClassification::MotorcycleMaintenance => {
                if let Ok(maint) = extract_maintenance_data(&state, &image_url).await {
                    media_context = format!(
                        "
[SYSTEM: User uploaded motorcycle maintenance record. Parts: {}. Details: {}]",
                        maint.part_names.join(", "), maint.service_details
                    );
                    let memory_content = format!("Motorcycle Maintenance: {} - Parts: {}", maint.service_details, maint.part_names.join(", "));
                    if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &memory_content).await {
                        let metadata = json!({
                            "type": "memory",
                            "source": "image_classification",
                            "classification": "MOTORCYCLE_MAINTENANCE",
                            "graph": {
                                "nodes": maint.part_names.iter().map(|p| json!({"id": p.to_lowercase().replace(' ', "_"), "label": p, "node_type": "MaintenanceLog"})).collect::<Vec<_>>(),
                                "links": maint.part_names.iter().map(|p| json!({"source": "motorcycle", "target": p.to_lowercase().replace(' ', "_"), "relationship": "replaced_part"})).collect::<Vec<_>>()
                            },
                            "data": maint,
                            "image_url": image_url
                        });
                        let _ = rag::save_to_knowledge_base(&state.pool, &memory_content, embedding, Some(metadata), Some(conversation_id)).await;
                    }
                }
            }
            MediaClassification::TechnicalDoc => {
                if let Ok(content) = extract_technical_doc(&state, &image_url).await {
                     media_context = format!("
[SYSTEM: User uploaded a technical document. Summary: {}]", 
                        if content.len() > 100 { &content[..100] } else { &content });
                     if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &content).await {
                        let metadata = json!({
                            "type": "memory",
                            "source": "image_classification",
                            "classification": "TECHNICAL_DOC",
                            "image_url": image_url
                        });
                        let _ = rag::save_to_knowledge_base(&state.pool, &content, embedding, Some(metadata), Some(conversation_id)).await;
                    }
                }
            }
            MediaClassification::Nature => { media_context = "
[SYSTEM: User uploaded a nature photo.]".to_string(); }
            MediaClassification::Other => { media_context = "
[SYSTEM: User uploaded an image (uncategorized).]".to_string(); }
        }

        // Save last_image_url to conversation metadata for just-in-time sticker generation
        let _ = sqlx::query!(
            "UPDATE conversations SET metadata = COALESCE(metadata, '{}'::jsonb) || jsonb_build_object('last_image_url', $1::text) WHERE id = $2",
            image_url,
            conversation_id
        ).execute(&state.pool).await;
    }

    let augmented_text = format!("{}{}", text_content, media_context);

    let dispatcher = ToolDispatcher::new(
        state.pool.clone(),
        std::env::current_dir().unwrap_or_default(),
        user_id.clone(),
        Some(conversation_id),
        state.gemini.clone(),
        state.gemini_api_key.clone(),
        state.sse.clone(),
    );

    let conversation = sqlx::query!(
        "SELECT bootstrap_content, soul_content FROM conversations WHERE id = $1",
        conversation_id
    )
    .fetch_one(&state.pool)
    .await?;

    let system_prompt = {
        let boot = conversation.bootstrap_content.unwrap_or_default();
        let soul = conversation.soul_content.unwrap_or_default();
        let mut combined = boot;
        if !soul.is_empty() {
            combined.push_str("

### Current Personality/Soul
");
            combined.push_str(&soul);
        }
        combined.push_str("

### Orchestrator Instructions
You are operating in a multi-turn tool-use loop. You MUST wait to gather all necessary data from your tools before providing a final response to the user. Do not answer prematurely. Acknowledge and integrate all tool results into your final answer.

**Direct Messaging Flow:**
- If a user says 'Tell [Name] [Message]', FIRST use `search_users` to find the correct JID.
- If `search_users` returns multiple results, ask the user for clarification (e.g., 'I found two Billys. Did you mean Billy the Rider or Billy the Coder?').
- Once the unique JID is identified, use `send_direct_message(recipient_jid, content)`.
- After sending, confirm to the sender: 'Done! I've sent that message to [Name]. 🚀'

**Sticker Generation:**
- If a user asks to turn an image into a sticker (e.g., 'Make this a sticker', 'Sticker-in', 'Jadikan sticker'), use the `make_sticker` tool.
- If no URL is provided, the tool will automatically use the most recent image from the conversation.");
        combined
    };

    let history = sqlx::query!(
        "SELECT users.display_name as display_name, messages.created_at, messages.role, messages.content FROM messages LEFT JOIN users ON users.id = messages.user_id WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 15",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await?;

    let mut history_text = String::new();
    for msg in history.into_iter().rev() {
        let role_label = match msg.role.as_str() {
            "user" => match msg.display_name {
                None => "User".to_string(),
                Some(ref user) => user.clone(),
            },
            "assistant" => "Nomi".to_string(),
            _ => "System".to_string(),
        };
        history_text.push_str(&format!(
            "-[{}] {}: {}.
",
            msg.created_at.unwrap_or(Utc::now()).format("%Y-%m-%d %H:%M").to_string(),
            role_label,
            msg.content
        ));
    }

    let embedding = rag::get_embedding(&state.gemini_api_key, &augmented_text)
        .await
        .unwrap_or_default();
    let memories_text = if !embedding.is_empty() {
        crate::utils::rag::hybrid_retrieve(
            &state.pool,
            &augmented_text,
            embedding,
            Some(conversation_id),
        )
        .await
        .unwrap_or_default()
        .join("
---
")
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
        send_status_update(&state.pool, conversation_id, "Nomi is thinking...".to_string());

        let result = crate::common::agent::send_prompt(state.gemini.as_ref(), current_actor).await;

        match result {
            Ok((response, chunk)) => {
                let mut turn_text = String::new();
                if !chunk.thought.is_empty() {
                    turn_text.push_str(&chunk.thought);
                    turn_text.push_str("
");
                    
                    accumulated_thought.push_str(&chunk.thought);
                    accumulated_thought.push_str("
");

                    let payload = json!({ "thought": chunk.thought, "conversation_id": conversation_id });
                    let _ = match user_id {
                        None => state.broadcast_sse("thought", payload).await,
                        Some(ref id) => state.send_sse_to_user(id.to_string().as_str(), "thought", payload).await
                    };
                }
                if !chunk.content.is_empty() {
                    turn_text.push_str(&chunk.content);
                    
                    accumulated_content.push_str(&chunk.content);
                    accumulated_content.push_str("

");
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

                if tool_calls.is_empty() && (finish_reason.contains("Stop") || finish_reason.is_empty()) {
                    let mut final_chunk = chunk.clone();
                    final_chunk.content = strip_thinking_tags(&accumulated_content).trim().to_string();
                    final_chunk.thought = accumulated_thought.trim().to_string();
                    final_chunk.prompt_tokens = total_prompt_tokens;
                    final_chunk.answer_tokens = total_answer_tokens;
                    final_chunk.total_tokens = total_tokens;

                    final_response = Some((response, final_chunk));
                    break;
                }

                if loop_count >= max_loops {
                    let mut final_chunk = chunk.clone();
                    final_chunk.content = strip_thinking_tags(&accumulated_content).trim().to_string();
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
                    let action = match call.name.as_str() {
                        "read_workspace_file" | "execute_read_query" | "parse_to_json" => format!("checking {}", call.name),
                        "web_search" | "read_web_page" => "searching the web".to_string(),
                        "update_conversation_soul" | "update_nomi_soul" => "updating soul".to_string(),
                        "update_knowledge_base" => "updating memory".to_string(),
                        "evolve_bootstrap" => "evolving".to_string(),
                        "create_reminder" | "modify_reminder" | "get_reminder_stats" => "managing reminders".to_string(),
                        "get_inbox_summary" => "checking your inbox".to_string(),
                        "send_direct_message" => "sending".to_string(),
                        "make_sticker" => "creating sticker".to_string(),
                        _ => format!("using {}", call.name),
                    };
                    send_status_update(&state.pool, conversation_id, format!("Nomi is {}...", action));
                }

                let tool_results = execute_tools(
                    &dispatcher,
                    current_calls.clone(),
                    &text_content, // use the v2-stripped one
                    Some(state.sse.clone()),
                ).await;

                // Append Tool Responses to history_text to enforce memory management persistence
                for (name, result) in &tool_results {
                    history_text.push_str(&format!(
                        "-[{}] System (Tool {} Result): {}.
",
                        Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                        name,
                        if result.success { &result.content } else { &result.error }
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
        if let Ok(record) = save_message(
            &state.pool,
            conversation_id,
            "assistant",
            &function_result.content,
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
        ).await {
            let payload = json!({
                "id": record.id,
                "conversation_id": conversation_id,
                "role": record.role,
                "content": record.content.clone(),
                "thought": record.thought,
                "user_id": record.user_id,
                "total_tokens": function_result.total_tokens,
                "created_at": record.created_at
            });

            let _ = match user_id {
                None => state.broadcast_sse("message", payload).await,
                Some(ref id) => state.send_sse_to_user(id.to_string().as_str(), "message", payload).await
            };

            let channel_info = sqlx::query!(
                "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
                conversation_id
            ).fetch_all(&state.pool).await.unwrap_or_default();

            for channel in channel_info {
                let outbound = OutboundMessage {
                    is_group: false,
                    sender_id: channel.external_id.clone(),
                    conversation_id: channel.external_chat_id.clone(),
                    text: record.content.clone(),
                    channel: channel.channel_type.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: None,
                };

                if let Ok(redis_url) = std::env::var("REDIS_URL") {
                    if let Ok(client) = redis::Client::open(redis_url) {
                        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                            use redis::AsyncCommands;
                            let payload = serde_json::to_string(&outbound).unwrap();
                            let _ = conn.publish::<&str, String, ()>("nomi:outbound", payload).await;
                        }
                    }
                }
            }
        }
        
        let pool = state.pool.clone();
        let gemini = state.gemini.clone();
        let gemini_api_key = state.gemini_api_key.clone();
        tokio::spawn(async move {
            let _ = trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id).await;
        });
    }

    let payload = json!({
        "conversation_id": conversation_id,
        "is_typing": false,
        "user_id": "nomi"
    });
    let _ = match user_id {
        None => state.broadcast_presence_sse(payload).await,
        Some(ref id) => state.send_presence_sse_to_user(id.to_string().as_str(), payload).await
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
            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        use redis::AsyncCommands;
                        let payload = serde_json::to_string(&presence).unwrap();
                        let _ = conn.publish::<&str, String, ()>("nomi:presence", payload).await;
                    }
                }
            }
        }
    }

    Ok(())
}