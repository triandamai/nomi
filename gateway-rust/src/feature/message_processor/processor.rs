use crate::AppState;
use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::media::{ExpenseData, MaintenanceData, MediaClassification};
use crate::feature::message_processor::model::UnifiedMessage;
use crate::feature::{InboundMessage, OutboundMessage, PresenceMessage};
use crate::rag;
use chrono::Utc;
use gemini_rust::{Content, Message};
use serde_json::json;
use uuid::Uuid;

use crate::common::repository::message_repo::save_message;
use crate::common::repository::{channel_repo, pairing_repo};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngExt;
use tracing::{error, info};

pub async fn process_incoming_message(state: AppState, msg: UnifiedMessage) -> anyhow::Result<()> {
    if msg.v2 {
        return crate::feature::message_processor::v2_orchestrator::process_v2_message(state, msg)
            .await;
    }

    let conversation_id = msg.conversation_id;
    let user_id = msg.user_id;
    let text_content = msg.text_content;

    info!(
        conversation_id = %conversation_id,
        user_id = ?user_id,
        source = ?msg.source,
        "Processing unified message"
    );

    // 1. Immediate Save
    let m = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id, created_at) VALUES ($1, 'user', $2, '', $3, now()) RETURNING id, role, content, thought, created_at, user_id",
        conversation_id,
        text_content,
        user_id
    ).fetch_one(&state.pool).await?;

    // Broadcast user message to SSE
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
        Some(ref id) => {
            state
                .send_sse_to_user(id.to_string().as_str(), "message", payload)
                .await
        }
    };

    // 2. Start Typing / Presence
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

    // Broadcast presence to Redis for channels
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

    // 2.5 Media Classification and Extraction
    let mut media_context = String::new();
    if let Some(image_url) = msg.image_url {
        info!("Media detected, classifying: {}", image_url);
        let classification = classify_media_context(&state, &image_url)
            .await
            .unwrap_or(MediaClassification::Other);
        info!("Media classified as: {:?}", classification);

        match classification {
            MediaClassification::ExpenseReceipt => {
                if let Ok(expense) = extract_expense_data(&state, &image_url).await {
                    media_context = format!(
                        "\n[SYSTEM: User uploaded an expense receipt. Merchant: {}, Total: {}, Category: {}. Items: {}]",
                        expense.merchant,
                        expense.total,
                        expense.category,
                        expense.items.join(", ")
                    );
                    // Save to Knowledge Base as memory
                    let memory_content = format!(
                        "Expense at {}: {} ({})",
                        expense.merchant, expense.total, expense.category
                    );
                    if let Ok(embedding) =
                        rag::get_embedding(&state.gemini_api_key, &memory_content).await
                    {
                        let metadata = json!({
                            "type": "memory",
                            "source": "image_classification",
                            "classification": "EXPENSE_RECEIPT",
                            "data": expense,
                            "image_url": image_url
                        });
                        let _ = rag::save_to_knowledge_base(
                            &state.pool,
                            &memory_content,
                            embedding,
                            Some(metadata),
                            Some(conversation_id),
                        )
                        .await;
                    }
                }
            }
            MediaClassification::MotorcycleMaintenance => {
                if let Ok(maint) = extract_maintenance_data(&state, &image_url).await {
                    media_context = format!(
                        "\n[SYSTEM: User uploaded motorcycle maintenance record. Parts: {}. Details: {}]",
                        maint.part_names.join(", "),
                        maint.service_details
                    );
                    let memory_content = format!(
                        "Motorcycle Maintenance: {} - Parts: {}",
                        maint.service_details,
                        maint.part_names.join(", ")
                    );
                    if let Ok(embedding) =
                        rag::get_embedding(&state.gemini_api_key, &memory_content).await
                    {
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
                        let _ = rag::save_to_knowledge_base(
                            &state.pool,
                            &memory_content,
                            embedding,
                            Some(metadata),
                            Some(conversation_id),
                        )
                        .await;
                    }
                }
            }
            MediaClassification::TechnicalDoc => {
                if let Ok(content) = extract_technical_doc(&state, &image_url).await {
                    media_context = format!(
                        "\n[SYSTEM: User uploaded a technical document. Summary: {}]",
                        if content.len() > 100 {
                            &content[..100]
                        } else {
                            &content
                        }
                    );
                    if let Ok(embedding) = rag::get_embedding(&state.gemini_api_key, &content).await
                    {
                        let metadata = json!({
                            "type": "memory",
                            "source": "image_classification",
                            "classification": "TECHNICAL_DOC",
                            "image_url": image_url
                        });
                        let _ = rag::save_to_knowledge_base(
                            &state.pool,
                            &content,
                            embedding,
                            Some(metadata),
                            Some(conversation_id),
                        )
                        .await;
                    }
                }
            }
            MediaClassification::Nature => {
                media_context = "\n[SYSTEM: User uploaded a nature photo.]".to_string();
            }
            MediaClassification::Other => {
                media_context = "\n[SYSTEM: User uploaded an image (uncategorized).]".to_string();
            }
        }
    }

    let augmented_text = format!("{}{}", text_content, media_context);

    // 3. Prepare AI Context
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
            combined.push_str("\n\n### Current Personality/Soul\n");
            combined.push_str(&soul);
        }
        combined
    };

    // History Retrieval
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
                None => "User",
                Some(ref user) => &user,
            },
            "assistant" => "Nomi",
            _ => "System",
        };
        history_text.push_str(&format!(
            "-[{}] {}: {}.\n",
            msg.created_at
                .unwrap_or(Utc::now())
                .format("%Y-%m-%d %H:%M")
                .to_string(),
            role_label,
            msg.content
        ));
    }

    // RAG Context
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
        .join("\n---\n")
    } else {
        String::new()
    };

    // 4. LLM Execution Loop
    let mut loop_count = 0;
    let max_loops = 5;
    let mut current_actor = PromptActor::User {
        history: history_text.clone(),
        memories: memories_text.clone(),
        message: augmented_text.clone(),
        system_prompt: system_prompt.clone(),
    };

    let mut final_response = None;
    let mut tool_turns = Vec::new();

    let mut accumulated_content = String::new();
    let mut accumulated_thought = String::new();
    let mut total_prompt_tokens = 0;
    let mut total_answer_tokens = 0;
    let mut total_tokens = 0;

    while loop_count < max_loops {
        loop_count += 1;
        info!("Loop count iterate(N): N({})", loop_count);
        let result = crate::common::agent::send_prompt(state.gemini.as_ref(), current_actor).await;

        match result {
            Ok((response, chunk)) => {
                // Accumulate turn data
                if !chunk.content.is_empty() {
                    accumulated_content.push_str(&chunk.content);
                    accumulated_content.push_str("\n\n");
                }
                if !chunk.thought.is_empty() {
                    accumulated_thought.push_str(&chunk.thought);
                    accumulated_thought.push_str("\n");

                    // Broadcast intermediate thought via SSE
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

                total_prompt_tokens += chunk.prompt_tokens;
                total_answer_tokens += chunk.answer_tokens;
                total_tokens += chunk.total_tokens;

                let tool_calls = response.function_calls();
                let finish_reason = chunk.finish_reason.clone().unwrap_or_default();

                // If no tool calls and model finished normally, we're done
                if tool_calls.is_empty()
                    && (finish_reason.contains("Stop") || finish_reason.is_empty())
                {
                    let mut final_chunk = chunk.clone();
                    final_chunk.content = accumulated_content.trim().to_string();
                    final_chunk.thought = accumulated_thought.trim().to_string();
                    final_chunk.prompt_tokens = total_prompt_tokens;
                    final_chunk.answer_tokens = total_answer_tokens;
                    final_chunk.total_tokens = total_tokens;

                    final_response = Some((response, final_chunk));
                    break;
                }

                // If we hit the loop limit, finalize with what we have
                if loop_count >= max_loops {
                    let mut final_chunk = chunk.clone();
                    final_chunk.content = accumulated_content.trim().to_string();
                    final_chunk.thought = accumulated_thought.trim().to_string();
                    final_chunk.prompt_tokens = total_prompt_tokens;
                    final_chunk.answer_tokens = total_answer_tokens;
                    final_chunk.total_tokens = total_tokens;

                    final_response = Some((response, final_chunk));
                    break;
                }

                let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();

                let tool_results = execute_tools(
                    &dispatcher,
                    current_calls.clone(),
                    &text_content,
                    Some(state.sse.clone()),
                )
                .await;

                tool_turns.push((current_calls, tool_results));

                current_actor = PromptActor::MultiTool {
                    history: history_text.clone(),
                    memories: memories_text.clone(),
                    message: text_content.clone(),
                    system_prompt: system_prompt.clone(),
                    tool_turns: tool_turns.clone(),
                };
            }
            Err(e) => {
                error!("Agentic loop error: {}", e);
                break;
            }
        }
    }

    // 5. Post-Process & Final Save
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
        )
        .await
        {
            // Broadcast assistant message to SSE
            let payload = json!({
                        "id": record.id,
                        "conversation_id":conversation_id,
                        "role": record.role,
                        "content": record.content.clone(),
                        "thought": record.thought,
                        "user_id": record.user_id,
                        "total_tokens": function_result.total_tokens,
                        "created_at": record.created_at
            });

            let _ = match user_id {
                None => state.broadcast_sse("message", payload).await,
                Some(ref id) => {
                    state
                        .send_sse_to_user(id.to_string().as_str(), "message", payload)
                        .await
                }
            };

            // Outbound Routing for Channels
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
                            let _ = conn
                                .publish::<&str, String, ()>("nomi:outbound", payload)
                                .await;
                        }
                    }
                }
            }
        }
        // Memory Consolidation Trigger (Background)
        let pool = state.pool.clone();
        let gemini = state.gemini.clone();
        let gemini_api_key = state.gemini_api_key.clone();
        tokio::spawn(async move {
            let _ =
                trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id).await;
        });
    }

    // Stop Typing
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

    // Presence Outbound (Stop Typing)
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

pub(crate) async fn trigger_memory_consolidation(
    pool: sqlx::PgPool,
    gemini: std::sync::Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    conversation_id: Uuid,
) -> anyhow::Result<()> {
    // 1. Get the last summarized message ID
    let last_summary = sqlx::query!(
        r#"
        SELECT metadata->>'last_message_id' as last_message_id
        FROM knowledge_base
        WHERE metadata->>'type' = 'summary' 
        AND metadata->>'conversation_id' = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        conversation_id.to_string()
    )
    .fetch_optional(&pool)
    .await?;

    let last_msg_id = last_summary
        .and_then(|r| r.last_message_id)
        .and_then(|id| Uuid::parse_str(&id).ok());

    // 2. Fetch new messages
    let new_messages = sqlx::query!(
        r#"
        SELECT id, role, content 
        FROM messages 
        WHERE conversation_id = $1 
        AND ($2::uuid IS NULL OR created_at > (SELECT created_at FROM messages WHERE id = $2))
        ORDER BY created_at ASC
        "#,
        conversation_id,
        last_msg_id
    )
    .fetch_all(&pool)
    .await?;

    // 3. Threshold check
    if new_messages.len() >= 10 {
        info!(conversation_id = %conversation_id, "Memory consolidation triggered ({} new messages)", new_messages.len());

        let last_processed_id = new_messages.last().map(|m| m.id).unwrap();
        let mut summary_input = String::new();
        for msg in new_messages {
            summary_input.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        let summarizer_prompt = format!(
            "Analyze the following conversation and return a JSON object with:\n
                1. 'summary': A concise summary of permanent facts and project context.\n
                2. 'nodes': An array of entities ({{'id': 'unique_id', 'label': 'Entity Name', 'node_type': 'Technology|Project|Person|Organization|Vehicle|Location|Peak|Language|Framework|MaintenanceLog|Concept|Event'}}).\n
                3. 'edges': An array of relationships ({{'source': 'node_id', 'target': 'node_id', 'relationship': 'Description'}}).\n

            Rules:
                - NEVER create a node with id 'summary' or that represents the conversation summary itself.\n
                - Extract individual entities.\n
                - Reuse IDs.\n
                - 'id' should be snake_case.\n

            Conversation:\n
            {}
",
            summary_input
        );

        let summary_res = gemini
            .generate_content()
            .with_user_message(summarizer_prompt)
            .execute()
            .await?;

        let raw_json = summary_res.text();
        let parsed_data: serde_json::Value = if let Some(start) = raw_json.find('{') {
            if let Some(end) = raw_json.rfind('}') {
                serde_json::from_str(&raw_json[start..=end])
                    .unwrap_or(json!({ "summary": raw_json, "nodes": [], "edges": [] }))
            } else {
                json!({ "summary": raw_json, "nodes": [], "edges": [] })
            }
        } else {
            json!({ "summary": raw_json, "nodes": [], "edges": [] })
        };

        let summary_text = parsed_data["summary"]
            .as_str()
            .unwrap_or(&raw_json)
            .to_string();

        if let Ok(embedding) = rag::get_embedding(&gemini_api_key, &summary_text).await {
            let metadata = json!({
                "type": "summary",
                "conversation_id": conversation_id.to_string(),
                "last_message_id": last_processed_id.to_string(),
                "graph": {
                    "nodes": parsed_data["nodes"],
                    "links": parsed_data["edges"]
                }
            });

            rag::save_to_knowledge_base(
                &pool,
                &summary_text,
                embedding,
                Some(metadata),
                Some(conversation_id.clone()),
            )
            .await?;
            info!(conversation_id = %conversation_id, "Memory consolidation complete");
        }
    }

    Ok(())
}

pub(crate) async fn classify_media_context(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<MediaClassification> {
    let prompt = "Classify this image into exactly one of these categories: EXPENSE_RECEIPT, MOTORCYCLE_MAINTENANCE, TECHNICAL_DOC, NATURE, or OTHER. Return ONLY the category name.";

    let (mime_type, base64_data) = fetch_image_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    let text = res.text().trim().to_uppercase();
    if text.contains("EXPENSE_RECEIPT") {
        Ok(MediaClassification::ExpenseReceipt)
    } else if text.contains("MOTORCYCLE_MAINTENANCE") {
        Ok(MediaClassification::MotorcycleMaintenance)
    } else if text.contains("TECHNICAL_DOC") {
        Ok(MediaClassification::TechnicalDoc)
    } else if text.contains("NATURE") {
        Ok(MediaClassification::Nature)
    } else {
        Ok(MediaClassification::Other)
    }
}

pub(crate) async fn extract_expense_data(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<ExpenseData> {
    let prompt = "Extract expense data from this receipt. Return a JSON object with: merchant, total (number), items (array of strings), and category. Return ONLY the JSON.";

    let (mime_type, base64_data) = fetch_image_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    let text = res.text();
    let json_str = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text.as_str()
        }
    } else {
        text.as_str()
    };

    let data: ExpenseData = serde_json::from_str(json_str)?;
    Ok(data)
}

pub(crate) async fn extract_maintenance_data(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<MaintenanceData> {
    let prompt = "Extract motorcycle maintenance data. Return a JSON object with: part_names (array of strings) and service_details. Return ONLY the JSON.";

    let (mime_type, base64_data) = fetch_image_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    let text = res.text();
    let json_str = if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            &text[start..=end]
        } else {
            text.as_str()
        }
    } else {
        text.as_str()
    };

    let data: MaintenanceData = serde_json::from_str(json_str)?;
    Ok(data)
}

pub(crate) async fn extract_technical_doc(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<String> {
    let prompt = "Summarize the content of this technical document. Focus on key specifications, diagrams, or instructions.";

    let (mime_type, base64_data) = fetch_image_from_storage(state, image_url).await?;

    let res = state
        .gemini
        .generate_content()
        .with_user_message(prompt)
        .with_message(Message {
            role: gemini_rust::Role::User,
            content: Content::inline_data(mime_type, base64_data),
        })
        .execute()
        .await?;

    Ok(res.text())
}

async fn fetch_image_from_storage(
    state: &AppState,
    image_url: &str,
) -> anyhow::Result<(String, String)> {
    let bucket = "conversations";
    // image_url from channel is typically just the filename/path in storage
    let data = state
        .storage
        .get_file(bucket.to_string(), image_url.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Storage error: {}", e))?;

    let mime_type = mime_guess::from_path(image_url)
        .first_or_octet_stream()
        .to_string();

    let b64 = BASE64.encode(data.to_vec());
    Ok((mime_type, b64))
}

pub async fn process_pairing(
    state: &AppState,
    msg: &InboundMessage,
    text: &str,
    user_id: Uuid,
) -> anyhow::Result<()> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() >= 2 {
        let code = parts[1].to_uppercase();
        if let Some(conv_id) = pairing_repo::validate_pairing_code(&state.pool, &code).await? {
            let display_name = match msg.metadata.clone() {
                None => None,
                Some(meta) => meta
                    .get("display_name")
                    .map_or_else(|| None, |v| Some(v.to_string())),
            };

            pairing_repo::complete_pairing(&state.pool, &code, user_id).await?;
            channel_repo::link_channel(
                &state.pool,
                &msg.channel,
                &msg.sender_id,
                &msg.conversation_id,
                conv_id,
                user_id,
                display_name,
            )
            .await?;

            let _ = state
                .send_to_user(
                    user_id.to_string().as_str(),
                    "pairing_success",
                    serde_json::json!({
                        "conversation_id": conv_id,
                        "platform": msg.channel,
                        "message": format!("Successfully paired with {}!", msg.channel)
                    }),
                    &OutboundMessage {
                        is_group: msg.is_group,
                        sender_id: msg.sender_id.clone(),
                        conversation_id: msg.conversation_id.clone(),
                        text: "Pairing successful! This conversation is now linked.".to_string(),
                        channel: msg.channel.clone(),
                        video_url: None,
                        image_url: None,
                        audio_url: None,
                        doc_url: None,
                        sticker_url: None,
                        metadata: msg.metadata.clone(),
                    },
                )
                .await;

            return Ok(());
        }
    }
    Ok(())
}

pub async fn process_register(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start registering from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );
    
    if msg.is_group {
        
        return Ok(());
    }
    let channel_exists = sqlx::query!("SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",msg.channel,msg.conversation_id)
        .fetch_optional(&state.pool)
        .await;
    if let Err(err) = channel_exists {
        info!("failed register because error getting information: {}", err);
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "We having trouble, meanwhile we on fixing, you can try again later."
                    .to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;
        return Ok(());
    }
    let channel_result = channel_exists?;
    if let Some(value) = channel_result {
        info!("failed register because user exist: {}", value.user_id);
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Account already exists. Use /login.".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            let _ = state
                .publish_outbond(&OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Internal server error".to_string(),
                    channel: msg.channel.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: msg.metadata.clone(),
                })
                .await;
            return Ok(());
        }
    };

    info!("begin create user \n");

    let display_name = match msg.metadata.clone() {
        None => msg.sender_id.clone(),
        Some(meta) => meta
            .get("display_name")
            .map_or_else(|| msg.sender_id.clone(), |v| v.to_string()),
    };

    let u_id = match sqlx::query!(
            "INSERT INTO users (external_id, display_name) VALUES ($1, $2) ON CONFLICT (external_id) DO UPDATE SET display_name = EXCLUDED.display_name RETURNING id",
            msg.sender_id,
            display_name
        ).fetch_one(&mut *tx).await {
        Ok(r) => r.id,
        Err(e) => {
            error!("Failed to resolve user: {}", e);
            let _ = tx.rollback().await;
            let _ = state.publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to resolve user".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            }).await;

            return Ok(());
        }
    };

    info!("begin create conversation \n");
    // Create new conversation
    let conv_id = Uuid::new_v4();
    let title = format!("{} via {}", msg.conversation_id, msg.channel);

    if let Err(e) = sqlx::query!(
        "INSERT INTO conversations (id, title) VALUES ($1, $2)",
        conv_id,
        title
    )
    .execute(&mut *tx)
    .await
    {
        error!("Failed to create conversation: {}", e);
        let _ = tx.rollback().await;
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to create conversation".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    info!("begin create channels");
    if let Err(e) = sqlx::query!(
            "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) VALUES ($1, $2, $3, $4, $5)",
            msg.channel,
            msg.sender_id,
            msg.conversation_id,
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to link channel: {}", e);
        let _ = tx.rollback().await;

        let _ = state.publish_outbond(&crate::feature::OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Failed to link channel".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        }).await;

        return Ok(());
    }

    if let Err(e) = sqlx::query!(
            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            conv_id,
            u_id
        ).execute(&mut *tx).await {
        error!("Failed to add member: {}", e);
        let _ = tx.rollback().await;
        let _ = state.publish_outbond(&crate::feature::OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Failed to join conversation".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        }).await;

        return Ok(());
    }

    if let Err(e) = tx.commit().await {
        error!("Failed to commit registration: {}", e);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Failed to register".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let _ = state
        .publish_outbond(&OutboundMessage {
            is_group: msg.is_group,
            sender_id: msg.sender_id.clone(),
            conversation_id: msg.conversation_id.clone(),
            text: "Success register account, you can now /login for access dashboard".to_string(),
            channel: msg.channel.clone(),
            video_url: None,
            image_url: None,
            audio_url: None,
            doc_url: None,
            sticker_url: None,
            metadata: msg.metadata.clone(),
        })
        .await;
    Ok(())
}

pub async fn process_login(state: &AppState, msg: &InboundMessage) -> anyhow::Result<()> {
    info!(
        "start login from channel {} sender_id {}",
        msg.channel, msg.sender_id
    );
    // Check if user/channel exists
    let channel_exists = sqlx::query!(
            "SELECT u.id as user_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            msg.channel,
            msg.conversation_id
        ).fetch_optional(&state.pool).await;

    if let Err(err) = channel_exists {
        info!("failed get channel data: {}", err);
        let _ = state
            .publish_outbond(&OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "We having trouble for getting information, meanwhile we fixing you can try again later.".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }
    if let Ok(None) = channel_exists {
        info!("channel doesnt exist:");
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Channel not registered, Use /register for new user use, if you already had account, get pairing code from dashboard and use /pair <PAIRING CODE>".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let channel_data = channel_exists.unwrap().unwrap();
    let user_id = channel_data.user_id;

    // Generate OTP
    let otp_code: u32 = rand::rng().random_range(100000..999999);
    let otp_str = otp_code.to_string();
    let redis_key = format!("otp:{}", user_id);

    if let Err(e) = state.redis.set_ex(&redis_key, &otp_str, 300).await {
        error!("Failed to store OTP in Redis: {}", e);
        let _ = state
            .publish_outbond(&crate::feature::OutboundMessage {
                is_group: msg.is_group,
                sender_id: msg.sender_id.clone(),
                conversation_id: msg.conversation_id.clone(),
                text: "Database error".to_string(),
                channel: msg.channel.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: None,
                metadata: msg.metadata.clone(),
            })
            .await;

        return Ok(());
    }

    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    let login_url = format!("{}/login?id={}", app_url, user_id);

    let outbound_text = format!(
        "Your verification code is: {}\n\nClick here to login: {}",
        otp_str, login_url
    );

    let outbound = crate::feature::OutboundMessage {
        is_group: msg.is_group,
        sender_id: "nomi_auth".to_string(),
        conversation_id: msg.conversation_id.clone(),
        text: outbound_text,
        channel: msg.channel.clone(),
        video_url: None,
        image_url: None,
        audio_url: None,
        doc_url: None,
        sticker_url: None,
        metadata: msg.metadata.clone(),
    };

    if let Err(e) = state.redis.publish_event("nomi:outbound", &outbound).await {
        error!("Failed to publish OTP to nomi:outbound: {}", e);
        return Ok(());
    }

    Ok(())
}
