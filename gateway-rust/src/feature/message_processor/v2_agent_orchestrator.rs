use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::execute_tools;
use crate::common::identity::UserIdentity;
use crate::common::repository::message_repo::save_message;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::v2_orchestrator::{
    send_message_to_subscriber, send_status_presence_update, send_status_update, send_tool_update,
};
use crate::feature::{Conversation, MessageSource, UnifiedMessage};
use crate::rag::trigger_memory_consolidation;
use crate::services::event_dispatcher::AppEvent;
use crate::services::intent_classifier::{ClassificationResult, IntentClassifierService};
use crate::{AppState, rag};
use anyhow::anyhow;
use chrono::Utc;
use rust_decimal::prelude::ToPrimitive;
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

pub struct V2AgentOrchestrator {
    pub state: AppState,
    pub current_user: Option<UserIdentity>,
    pub conversation: Option<Conversation>,
    pub conversation_member_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionTrigger {
    UserRequested { reason: String },
    ProactiveCheck { reason: String },
    SystemAlert { reason: String },
}

impl V2AgentOrchestrator {
    pub fn new(
        state: AppState,
        conversation: Option<Conversation>,
        current_user: Option<UserIdentity>,
        conversation_member_ids: Vec<Uuid>,
    ) -> Self {
        Self {
            state,
            current_user,
            conversation,
            conversation_member_ids,
        }
    }

    pub async fn process_v2_message_with_intent(
        &self,
        state: AppState,
        msg: UnifiedMessage,
        text_content: String,
        _injected_system_prompt: Option<String>,
    ) -> anyhow::Result<()> {
        let conversation_id = msg.conversation_id;
        let user_id = msg.user_id;

        info!(
            conversation_id = %conversation_id,
            user_id = ?user_id,
            "Processing v2 message loop with intent"
        );

        // Broadcast initial presence (Typing: ON)
        send_status_presence_update(
            &state,
            self.conversation_member_ids.clone(),
            conversation_id,
            msg.source.clone(),
            msg.is_group,
            true,
        )
        .await;

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
                .dispatch(AppEvent::conversation(
                    conversation_id,
                    "token_update",
                    serde_json::json!({
                        "conversation_id": conversation_id,
                        "cumulative_tokens": row.cumulative_tokens.unwrap_or(0).to_u64().unwrap_or(0)
                    }),
                ))
                .await;
        }

        let _ = send_status_presence_update(
            &state,
            self.conversation_member_ids
                .iter()
                .map(|v| v.clone())
                .collect(),
            conversation_id,
            msg.source.clone(),
            msg.is_group,
            true,
        );

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
        for msg_h in history.into_iter().rev() {
            let is_processed = if let Some(meta) = msg_h.metadata {
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

            let image_url = match msg_h.image_url {
                Some(path) => {
                    let status = if is_processed {
                        "[ALREADY PROCESSED]"
                    } else {
                        "[PENDING ACTION]"
                    };
                    format!(
                        " - Image URL: {} {} \n",
                        state.storage.get_full_url(&path),
                        status
                    )
                }
                _ => "".to_string(),
            };

            let role_label = match msg_h.role.as_str() {
                "user" => match msg_h.display_name {
                    None => "User".to_string(),
                    Some(ref user) => user.clone(),
                },
                "assistant" => "Nomi".to_string(),
                _ => "System".to_string(),
            };

            history_text.push_str(&format!(
                "-[{}] {}: {}.{}\n",
                msg_h
                    .created_at
                    .unwrap_or(Utc::now())
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
                role_label,
                msg_h.content,
                image_url
            ));
        }

        let dispatcher = ToolDispatcher::new(
            state.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
            user_id.clone(),
            Some(conversation_id),
            state.gemini.clone(),
            state.gemini_api_key.clone(),
            state.storage.clone(),
            state.clone(),
        );

        let classifier = IntentClassifierService::new();
        let intent = classifier
            .classify_user_intent(
                &dispatcher.clone(),
                msg.text_content.clone().as_str(),
                history_text.as_str(),
            )
            .await
            .map_or_else(
                |_| ClassificationResult {
                    intents: vec![],
                    input_tokens: 0,
                    output_tokens: 0,
                    total_tokens: 0,
                },
                |v| v,
            );

        let conversation = self.conversation.clone().unwrap();

        let incoming_ctx = json!({
            "is_group": msg.is_group,
            "is_mentioned": msg.is_mentioned,
            "sender_id": msg.user_id,
            "conversation_id": msg.conversation_id,
            "text": msg.text_content,
            "channel": match &msg.source {
                MessageSource::Web { name } => name.clone(),
                MessageSource::Telegram { name } => name.clone(),
                MessageSource::WhatsApp { name } => name.clone(),
                MessageSource::Other { name } => name.clone(),
                MessageSource::Multiple { source } => source.join(", "),
            },
            "image_url": msg.image_url,
            "video_url": msg.video_url,
            "audio_url": msg.audio_url,
            "doc_url": msg.doc_url,
            "sticker_url": msg.sticker_url,
        });

        let workspace_ctx = json!({
            "id": conversation.id,
            "title": conversation.title
        });

        let mut intents = intent.intents.clone();

        info!("Scout Intents Detected: {:?}", intents);
        let old_system_prompt_len = {
            let boot = conversation.bootstrap_content.clone().unwrap_or_default();
            let soul = conversation.soul_content.clone().unwrap_or_default();
            boot.len()
                + soul.len()
                + crate::prompts::PromptRegistry::orchestrator_instructions().len()
                + crate::prompts::PromptRegistry::tool_usage_guidelines().len()
        };

        let build_system_prompt = |intents_val: &[String]| -> String {
            let mut combined = String::new();
            combined.push_str(crate::prompts::PromptRegistry::CORE_RULES);
            combined.push_str(crate::prompts::PromptRegistry::BOUNDARIES);

            let boot = conversation.bootstrap_content.clone().unwrap_or_default();
            let soul = conversation.soul_content.clone().unwrap_or_default();

            combined.push_str("\n### Identity Layer\n");
            combined.push_str(&boot);

            // [FIX] Visual Context Injection
            // Fetch the latest unprocessed media directly from history and inject it as context.
            let pending_media = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    crate::common::repository::message_repo::get_latest_unprocessed_media(
                        &state.pool,
                        conversation_id,
                    )
                    .await
                    .ok()
                    .flatten()
                })
            });

            if let Some((url, _type)) = pending_media {
                let full_url = state.storage.get_full_url(&url);
                combined.push_str("\n");
                combined.push_str(&format!(
                    "### ACTIVE VISUAL BUFFER\n- Current File: {}\n- Instruction: This file is currently 'Active' and ready for tools like `create_sticker` or `log_expense`. Use the URL provided here for the tool call if the user's intent matches.\n",
                    full_url
                ));
            }

            if !soul.is_empty() {
                combined.push_str("\n### Current Personality/Soul\n");
                combined.push_str(&soul);
            }

            let timezone_str = "Asia/Jakarta";
            let tz: chrono_tz::Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);
            let now_local = Utc::now().with_timezone(&tz);

            combined.push_str(&format!(
                "\n### Current Contextual Time\n- UTC: {}\n- Local Time: {} ({})\n",
                Utc::now().to_rfc3339(),
                now_local.to_rfc3339(),
                timezone_str
            ));

            combined.push_str("\n### Timezone & Tool Parameter Instructions\n");
            combined.push_str(&format!(
                "The user's current local time is {} (Asia/Jakarta). \n\
                 When calling date-range tracking tools like `get_reminder_stats`, you MUST format parameters like `start_after` and `end_before` as absolute strict ISO 8601 strings with offsets.\n\
                 For a query about 'today', start_after MUST be formatted exactly as '{}-00:00:00+07:00' and end_before as '{}-23:59:59+07:00'.\n",
                now_local.format("%H:%M"),
                now_local.format("%Y-%m-%d"),
                now_local.format("%Y-%m-%d")
            ));

            combined.push_str("\n### Orchestrator Instructions \n");
            combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());

            if !intents_val.contains(&"GENERAL".to_string()) || intents_val.len() > 1 {
                // Modular Domain Logic from Plugins
                let mut domain_rules = String::new();
                for plugin in dispatcher.plugins.values() {
                    let plugin_intents = plugin.matching_intents();
                    if intents_val
                        .iter()
                        .any(|i| plugin_intents.contains(&i.as_str()))
                        || intents_val.contains(&"FULL_REGISTRY".to_string())
                    {
                        let rules = plugin.rules();
                        if !rules.is_empty() && !domain_rules.contains(rules) {
                            domain_rules.push_str(rules);
                        }
                    }
                }
                combined.push_str(&domain_rules);
            }

            if intents_val.contains(&"FULL_REGISTRY".to_string()) {
                combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());
            }

            combined
        };

        let mut system_prompt = build_system_prompt(&intents);

        let new_system_prompt_len = system_prompt.len();
        let saved_percent = if old_system_prompt_len > 0 {
            ((old_system_prompt_len as f64 - new_system_prompt_len as f64)
                / old_system_prompt_len as f64
                * 100.0)
                .max(0.0)
        } else {
            0.0
        };
        info!("Tokens saved by modular assembly: {:.2}%", saved_percent);

        let embedding = rag::get_embedding(&state.gemini_api_key, &text_content).await;

        // --- 🚀 Step 1: Dynamic Context Pruning via Intent Classifier 🚀 ---
        // Safety: Only bypass RAG if it's pure chitchat without complex entities or questions
        let is_pure_chitchat = (intents.contains(&"CHITCHAT".to_string())
            || intents.contains(&"GENERAL".to_string()))
            && intents.len() == 1;

        // Simple entity check: contains non-alphanumeric chars (excluding space) often indicates technical data or complex names
        let has_potential_entities = text_content
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace() && c != '?' && c != '!' && c != '.');
        let is_question = text_content.trim().ends_with('?');

        let memories_text = if is_pure_chitchat && !has_potential_entities && !is_question {
            info!("Pure chitchat detected (intent: {:?}). Bypassing RAG retrieval to save context tokens.", intents);
            String::new()
        } else if embedding.is_ok() {
            crate::utils::rag::hybrid_retrieve(
                &state.pool,
                &text_content,
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

        // [FIX] Proper Async Fetch for Pending Media
        let pending_media = crate::common::repository::message_repo::get_latest_unprocessed_media(
            &state.pool,
            conversation_id,
        )
        .await
        .ok()
        .flatten();

        // [NEW] Fetch raw media bytes with robust path/mime handling
        let mut raw_media = None;
        if let Some((url, _type)) = pending_media.as_ref() {
            let base_url = dotenvy::var("PUBLIC_GATEWAY_URL")
                .unwrap_or("http://localhost:8000/api".to_string());
            let file_path = if url.starts_with("http") && url.contains(&base_url) {
                url.replace(&format!("{}/files/", base_url), "")
            } else {
                url.clone()
            };

            if let Ok(data) = state
                .storage
                .get_file("conversations".to_string(), file_path.clone())
                .await
            {
                let mime_type = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();

                // Gemini rejects generic octet-stream. Force image fallbacks for multimodal safety.
                let safe_mime = if mime_type == "application/octet-stream" {
                    if file_path.to_lowercase().ends_with(".png") {
                        "image/png".to_string()
                    } else if file_path.to_lowercase().ends_with(".webp") {
                        "image/webp".to_string()
                    } else {
                        "image/jpeg".to_string()
                    }
                } else {
                    mime_type
                };

                use base64::Engine;
                let base64_data = base64::engine::general_purpose::STANDARD.encode(data.to_vec());
                raw_media = Some((safe_mime, base64_data));
                info!(
                    "Multimodal: Prepared media context (mime: {})",
                    raw_media.as_ref().unwrap().0
                );
            }
        }

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
        let mut has_retried = false;

        while loop_count < max_loops {
            loop_count += 1;
            info!("V2 Loop iterate(N): N({})", loop_count);

            // Multimodal Rule: Only send the heavy media bytes on the FIRST turn.
            // Downstream turns will rely on the conversation history and the agent's memory of the image.
            let media_to_send = if loop_count == 1 {
                raw_media.clone()
            } else {
                None
            };

            let current_actor = if loop_count == 1 {
                PromptActor::User {
                    history: history_text.clone(),
                    memories: memories_text.clone(),
                    message: text_content.clone(),
                    system_prompt: system_prompt.clone(),
                    media: media_to_send,
                }
            } else {
                PromptActor::MultiTool {
                    history: history_text.clone(),
                    memories: memories_text.clone(),
                    message: text_content.clone(),
                    system_prompt: system_prompt.clone(),
                    tool_turns: tool_turns.clone(),
                    media: media_to_send,
                }
            };

            // Status: Model is thinking
            if loop_count <= 1 {
                let _ = send_status_update(
                    &state,
                    self.conversation_member_ids
                        .iter()
                        .map(|v| v.clone())
                        .collect(),
                    conversation_id,
                    MessageSource::Web {
                        name: "web".to_string(),
                    },
                    msg.is_group,
                    "thought".to_string(),
                    crate::prompts::StatusRegistry::random_thinking_phrase(),
                )
                .await;
            }

            // --- 🚀 Step 2: Shape Persona Velocity via InteractionGate 🚀 ---
            // Adjusting token output depth to feel casually human
            let is_short_message = text_content.len() < 30;
            let (_temp, _max_tokens) = if is_short_message && !msg.is_group {
                (0.8, Some(128)) // Higher temperature for casual warmth; small token cap
            } else {
                (0.2, None) // Low temperature for precision; standard cap (None uses default)
            };

            // Implementation Note: We are currently using the default Gemini configuration.
            // Future refinement: Add dynamic parameter support to gemini-rust or ToolDispatcher.
            
            let result =
                crate::common::agent::send_prompt(&dispatcher, current_actor, &intents).await;

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
                            None => {
                                state
                                    .dispatch(AppEvent::broadcast("thought", payload))
                                    .await
                            }
                            Some(ref id) => {
                                state
                                    .dispatch(AppEvent::user(
                                        id.to_string().as_str(),
                                        "thought",
                                        payload,
                                    ))
                                    .await
                            }
                        };
                    }
                    if !chunk.content.is_empty() {
                        // Task 2: Recursive Retry on Truncation
                        if chunk.content.trim().ends_with(':') && chunk.content.len() < 100 {
                            info!(
                                "Truncation detected (ends with ':'). Triggering self-correction."
                            );
                            history_text.push_str(&format!(
                                "-[{}] Nomi: {}.",
                                Utc::now().format("%Y-%m-%d %H:%M").to_string(),
                                chunk.content
                            ));
                            history_text.push_str(&format!(
                                "-[{}] System: Continue your response starting from the code block.",
                                Utc::now().format("%Y-%m-%d %H:%M").to_string()
                            ));
                            continue;
                        }

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
                        let current_content_is_empty =
                            strip_thinking_tags(&chunk.content).trim().is_empty();
                        if !tool_turns.is_empty()
                            && current_content_is_empty
                            && loop_count < max_loops
                        {
                            info!(
                                "Synthesis Turn: Model tried to stop after tools without content. Forcing synthesis turn."
                            );
                            // Force history update to reflect tool results were seen
                            history_text.push_str(&format!(
                                "-[{}] System: Please synthesize the tool results into a final response for the user. Do not call the same tools again.",
                                Utc::now().format("%Y-%m-%d %H:%M").to_string()
                            ));
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

                    // --- 🚀 Step 1: Fix Content Accumulation 🚀 ---
                    if !current_calls.is_empty() && !finish_reason.eq_ignore_ascii_case("stop") {
                        // If the model is calling tools and not yet finished, 
                        // clear the teaser text ("Let me check...") to prevent duplication in final answer.
                        accumulated_content.clear();
                    }

                    // Status: Tool checking
                    for call in &current_calls {
                        let _ = send_tool_update(
                            &state,
                            self.conversation_member_ids
                                .iter()
                                .map(|v| v.clone())
                                .collect(),
                            conversation_id,
                            MessageSource::Web {
                                name: "web".to_string(),
                            },
                            msg.is_group,
                            "tool_start".to_string(),
                            call.name.clone(),
                        )
                        .await;
                    }

                    let tool_results = execute_tools(
                        &dispatcher,
                        current_calls.clone(),
                        incoming_ctx.clone(),
                        workspace_ctx.clone(),
                    )
                    .await;

                    // --- 🚨 START DISCOVERY INTERCEPTION HOOK 🚨 ---
                    let mut discovered_intents = Vec::new();
                    for (name, result) in &tool_results {
                        if name == "discover_tools" && result.success {
                            if let Some(start_idx) = result.content.find("[INTENT_INJECTION:") {
                                if let Some(end_idx) = result.content[start_idx..].find(']') {
                                    let intents_raw =
                                        &result.content[start_idx + 18..start_idx + end_idx];
                                    for split_intent in intents_raw.split(',') {
                                        let trimmed = split_intent.trim().to_string();
                                        if !trimmed.is_empty() {
                                            discovered_intents.push(trimmed);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !discovered_intents.is_empty() {
                        info!(
                            "🔄 Orchestrator successfully captured missing intents: {:?}",
                            discovered_intents
                        );
                        for found_intent in discovered_intents {
                            if !intents.contains(&found_intent) {
                                intents.push(found_intent);
                            }
                        }
                        // Automatically re-load the system prompt definitions with the new domain rules included!
                        system_prompt = build_system_prompt(&intents);
                    }
                    // --- 🚨 END DISCOVERY INTERCEPTION HOOK 🚨 ---

                    let mut unknown_tool_called = false;
                    for (_, res) in &tool_results {
                        if res.error.starts_with("Unknown tool") {
                            unknown_tool_called = true;
                            break;
                        }
                    }

                    if unknown_tool_called && !has_retried {
                        info!(
                            "Scout error detected: Unknown tool called. Retrying with FULL_REGISTRY"
                        );
                        has_retried = true;
                        intents = vec!["FULL_REGISTRY".to_string()];
                        system_prompt = build_system_prompt(&intents);
                        loop_count -= 1; // Retry this iteration
                        continue;
                    }

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

                    let _ = send_status_update(
                        &state,
                        self.conversation_member_ids
                            .iter()
                            .map(|v| v.clone())
                            .collect(),
                        conversation_id,
                        MessageSource::Multiple {
                            source: vec!["web".to_string(), "mobile".to_string()],
                        },
                        false,
                        "error".to_string(),
                        "Oops, something went wrong.".to_string(),
                    )
                    .await;
                    break;
                }
            }
        }

        if let Some((_, function_result)) = final_response {
            // Double-check thinking tags are stripped (they already are above, but for clarity)
            let sanitized_content = strip_thinking_tags(&function_result.content)
                .trim()
                .to_string();

            let save_message = save_message(
                &state.pool,
                conversation_id,
                "assistant",
                &sanitized_content,
                Some(function_result.thought.as_str()),
                None,
                function_result.prompt_tokens + intent.input_tokens.to_i32().unwrap_or(0),
                function_result.answer_tokens + intent.output_tokens.to_i32().unwrap_or(0),
                function_result.total_tokens + intent.total_tokens.to_i32().unwrap_or(0),
                None,
                None,
                None,
                None,
                None,
            )
            .await;
            if let Err(err) = save_message{
                info!("Failed save message result {}",err);
                return Ok(());
            }
            if let Ok(record) = save_message {
                let member_ids = self.conversation_member_ids
                    .iter()
                    .map(|v| v.clone())
                    .collect();
                let _ = send_message_to_subscriber(
                    &state,
                    member_ids,
                    conversation_id,
                    msg.source.clone(),
                    record.to_sse_json(function_result.total_tokens),
                    record.clone(),
                )
                .await;

                // Fetch updated total tokens and broadcast
                if let Ok(row) = sqlx::query!(
                    "SELECT cumulative_tokens FROM conversations WHERE id = $1",
                    conversation_id
                )
                .fetch_one(&state.pool)
                .await
                {
                    let _ = state
                        .dispatch(AppEvent::conversation(
                            conversation_id,
                            "token_update",
                            json!({
                                "conversation_id": conversation_id,
                                "cumulative_tokens": row.cumulative_tokens
                            }),
                        ))
                        .await;
                }
            }

            let pool = state.pool.clone();
            let gemini = state.gemini.clone();
            let gemini_api_key = state.gemini_api_key.clone();
            tokio::spawn(async move {
                if let Ok((_convo_id, _total_token)) =
                    trigger_memory_consolidation(pool, gemini, gemini_api_key, conversation_id)
                        .await
                {}
            });

            let _ = send_status_presence_update(
                &state,
                self.conversation_member_ids
                    .iter()
                    .map(|v| v.clone())
                    .collect(),
                conversation_id,
                msg.source,
                msg.is_group,
                false,
            );
            return Ok(());
        }
        Ok(())
    }

    pub async fn process_background_job(
        &self,
        task_prompt: &str,
        trigger: ExecutionTrigger,
    ) -> anyhow::Result<String> {
        info!(
            " V2Orchestrator: Processing background job  conversation_id = {:?},  user_id = {:?}",
            self.conversation, self.current_user
        );

        if let None = self.conversation {
            info!("Workspace doesnt exist");
            return Err(anyhow!("Workspace doesnt exist"));
        }
        let conversation = self.conversation.clone().unwrap();
        let conversation_id = conversation.id;
        let workspace_soul = conversation.soul_content;
        let workspace_bootstrap = conversation.bootstrap_content;

        if let None = self.current_user {
            info!("Workspace doesnt exist");
            return Err(anyhow!("Workspace doesnt exist"));
        }
        let current_user = self.current_user.clone().unwrap();
        let user_id = current_user.id;

        let dispatcher = ToolDispatcher::new(
            self.state.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
            Some(user_id.clone()),
            Some(conversation_id.clone()),
            self.state.gemini.clone(),
            self.state.gemini_api_key.clone(),
            self.state.storage.clone(),
            self.state.clone(),
        );

        let build_system_prompt = |intents_val: &[String]| -> String {
            let mut combined = String::new();

            let timezone_str = "Asia/Jakarta";
            let tz: chrono_tz::Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);
            let now_local = Utc::now().with_timezone(&tz);

            // Situation Awareness Injection
            let (trigger_type, trigger_reason) = match &trigger {
                ExecutionTrigger::UserRequested { reason } => ("USER_REQUESTED", reason),
                ExecutionTrigger::ProactiveCheck { reason } => ("PROACTIVE_CHECK", reason),
                ExecutionTrigger::SystemAlert { reason } => ("SYSTEM_ALERT", reason),
            };

            combined.push_str(&format!(
                "[INTERNAL CONTEXT: This turn was triggered by a SCHEDULED TASK. Trigger Type: {}. Reason: {}. Current Local Time: {}.]\n",
                trigger_type,
                trigger_reason,
                now_local.to_rfc3339()
            ));

            combined.push_str(crate::prompts::PromptRegistry::CORE_RULES);
            combined.push_str(crate::prompts::PromptRegistry::BOUNDARIES);

            let boot = workspace_bootstrap.clone().unwrap_or_default();
            let soul = workspace_soul.clone().unwrap_or_default();

            combined.push_str("### Identity Layer");
            combined.push_str(&boot);
            if !soul.is_empty() {
                combined.push_str("### Current Personality/Soul");
                combined.push_str(&soul);
            }

            combined.push_str(&format!(
                "\
            ### Current Contextual Time\n
            - UTC: {}\n
            - Local Time: {} ({})\n",
                Utc::now().to_rfc3339(),
                now_local.to_rfc3339(),
                timezone_str
            ));

            combined.push_str("\n### Timezone Instructions]n");
            combined.push_str(&format!(
                "The user's current local time is {} ({}). When the user asks for a time like \"6:00\", assume they mean this local time and calculate the UTC equivalent for storage using the +07:00 offset or by converting from Asia/Jakarta. ALWAYS provide the due_at in ISO 8601 format including the local offset (e.g., {} ) for the tool call, but you can acknowledge the local time in your thoughts.\n",
                now_local.format("%H:%M"),
                timezone_str,
                now_local.format("%Y-%m-%dT%H:%M:%S%z")
            ));

            combined.push_str("\n### Orchestrator Instructions\n");
            combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());

            // Modular Domain Logic from Plugins
            let mut domain_rules = String::new();
            for plugin in dispatcher.plugins.values() {
                let plugin_intents = plugin.matching_intents();
                if intents_val
                    .iter()
                    .any(|i| plugin_intents.contains(&i.as_str()))
                    || intents_val.contains(&"FULL_REGISTRY".to_string())
                {
                    let rules = plugin.rules();
                    if !rules.is_empty() && !domain_rules.contains(rules) {
                        domain_rules.push_str(rules);
                    }
                }
            }
            combined.push_str(&domain_rules);

            if intents_val.contains(&"FULL_REGISTRY".to_string()) {
                combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());
            }

            combined
        };

        let intents_list = vec!["FULL_REGISTRY".to_string()];
        let system_prompt = build_system_prompt(&intents_list);

        let incoming_ctx = json!({
            "is_group": false,
            "is_mentioned": true,
            "sender_id": user_id,
            "conversation_id": conversation_id,
            "text": task_prompt,
            "channel": "system_task"
        });

        let workspace_ctx = json!({
            "id": conversation_id,
            "title": conversation.title
        });

        let mut loop_count = 0;
        let max_loops = 10;
        let mut tool_turns = Vec::new();
        let mut accumulated_content = String::new();
        let mut accumulated_thought = String::new();

        let mut total_prompt_tokens = 0;
        let mut total_answer_tokens = 0;
        let mut total_tokens = 0;

        while loop_count < max_loops {
            loop_count += 1;
            info!("V2Orchestrator Loop iterate(N): N({})", loop_count);

            let current_actor = PromptActor::MultiTool {
                history: "".to_string(),
                memories: "".to_string(),
                message: task_prompt.to_string(),
                system_prompt: system_prompt.clone(),
                tool_turns: tool_turns.clone(),
                media: None,
            };

            let result =
                crate::common::agent::send_prompt(&dispatcher, current_actor, &intents_list).await;

            match result {
                Ok((response, chunk)) => {
                    if !chunk.thought.is_empty() {
                        accumulated_thought.push_str(&chunk.thought);
                        accumulated_thought.push_str(
                            "
",
                        );
                    }
                    if !chunk.content.is_empty() {
                        accumulated_content.push_str(&chunk.content);
                        accumulated_content.push_str(
                            "
",
                        );
                    }

                    total_prompt_tokens += chunk.prompt_tokens;
                    total_answer_tokens += chunk.answer_tokens;
                    total_tokens += chunk.total_tokens;

                    let tool_calls = response.function_calls();
                    let finish_reason = chunk.finish_reason.clone().unwrap_or_default();

                    if tool_calls.is_empty()
                        && (finish_reason.eq_ignore_ascii_case("stop") || finish_reason.is_empty())
                    {
                        break;
                    }

                    if loop_count >= max_loops {
                        break;
                    }

                    let current_calls: Vec<_> = tool_calls.into_iter().map(|c| c.clone()).collect();
                    let tool_results =
                        execute_tools(&dispatcher, current_calls.clone(), incoming_ctx.clone(), workspace_ctx.clone()).await;

                    tool_turns.push((current_calls, tool_results));
                }
                Err(e) => {
                    error!("V2Orchestrator: Agentic loop error: {}", e);
                    return Err(anyhow::anyhow!(e));
                }
            }
        }

        let final_text = strip_thinking_tags(&accumulated_content);

        // Save and finalize
        let message = save_message(
            &self.state.pool,
            conversation_id,
            "assistant",
            &final_text,
            Some(&accumulated_thought),
            None,
            total_prompt_tokens,
            total_answer_tokens,
            total_tokens,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

        if let Ok(msg) = message {
            let _notify = send_message_to_subscriber(
                &self.state,
                self.conversation_member_ids
                    .iter()
                    .map(|v| v.clone())
                    .collect(),
                conversation_id,
                MessageSource::Multiple {
                    source: vec![
                        "whatsapp".to_string(),
                        "telegram".to_string(),
                        "other".to_string(),
                        "web".to_string(),
                    ],
                },
                msg.to_sse_json(total_tokens),
                msg,
            )
            .await;
        }

        // --- 🚨 FINAL PRESENCE UPDATE (OFF) 🚨 ---
        send_status_presence_update(
            &self.state,
            self.conversation_member_ids.clone(),
            conversation_id,
            MessageSource::Multiple {
                source: vec!["web".to_string(), "whatsapp".to_string(), "telegram".to_string()],
            },
            false, // is_group fallback
            false, // is_typing: OFF
        )
        .await;

        Ok(final_text)
    }
}

fn strip_thinking_tags(text: &str) -> String {
    let healed = crate::common::format::heal_thinking_tags(text);
    let re = regex::Regex::new(r"(?si)<thinking>.*?</thinking>|<thinking>.*").unwrap();
    re.replace_all(&healed, "").trim().to_string()
}
