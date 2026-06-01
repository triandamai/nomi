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
use crate::AppState;
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

        let retrieval = crate::rag::RagRetrieval::new(state.clone(), dispatcher.clone())
            .with_history(15);

        let mut history_text = retrieval.fetch_history().await?;

        let classifier = IntentClassifierService::new();
        let default_thresholds = json!({});
        let thresholds = self.conversation.as_ref()
            .and_then(|c| c.gateway_thresholds.as_ref())
            .unwrap_or(&default_thresholds);

        let intent = classifier
            .classify_user_intent(
                &dispatcher.clone(),
                msg.text_content.clone().as_str(),
                history_text.as_str(),
                thresholds,
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

        // --- 🚀 Step 1: Dynamic Context Pruning via Intent Classifier 🚀 ---
        // Safety: Only bypass RAG if it's pure chitchat without complex entities or questions
        let is_pure_chitchat = (intents.contains(&"CHITCHAT".to_string())
            || intents.contains(&"GENERAL".to_string()))
            && intents.len() == 1;

        // Simple entity check: contains non-alphanumeric chars (excluding space) often indicates technical data or complex names
        let has_potential_entities = msg.text_content
            .chars()
            .any(|c| !c.is_alphanumeric() && !c.is_whitespace() && c != '?' && c != '!' && c != '.');
        let is_question = msg.text_content.trim().ends_with('?');

        let memories_text = if is_pure_chitchat && !has_potential_entities && !is_question {
            info!("Pure chitchat detected (intent: {:?}). Bypassing RAG retrieval to save context tokens.", intents);
            String::new()
        } else {
            retrieval.clone().with_retrieval(msg.text_content.clone()).fetch_memories().await.unwrap_or_default()
        };

        // 🌟 AGENTIC DISCOVERY HOOK: If no specific domain intent is found, force-inject Discovery
        // This ensures every unrecognized user request passes through the v2 Blueprint Loop.
        if (intents.contains(&"CHITCHAT".to_string()) || intents.contains(&"GENERAL".to_string())) && intents.len() == 1 {
            info!("Capability gap detected. Force-injecting SYSTEM_INTERNAL_DISCOVERY for autonomous expansion.");
            intents.push("SYSTEM_INTERNAL_DISCOVERY".to_string());
        }

        info!("Scout Intents Detected: {:?}", intents);
        let old_system_prompt_len = {
            let boot = conversation.bootstrap_content.clone().unwrap_or_default();
            let soul = conversation.soul_content.clone().unwrap_or_default();
            boot.len()
                + soul.len()
                + crate::prompts::PromptRegistry::orchestrator_instructions().len()
                + crate::prompts::PromptRegistry::tool_usage_guidelines().len()
        };

        let mut system_prompt = retrieval
            .generate_system_prompt(&self.current_user, &conversation, &intents, &memories_text)
            .await?;

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

        // [FIX] Proper Async Fetch for Pending Media via universal RagRetrieval
        let raw_media = retrieval.fetch_raw_media().await;

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
        let mut accumulated_metadata = json!({});
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
                            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
                            history_text.push_str(&format!(
                                "- <MessageEntry timestamp=\"{}\" id=\"CORRECTION_TRUNCATION\" type=\"ASSISTANT_RETRY\">\n\
                                 \x20\x20\x20\x20[Actor]: Nomi\n\
                                 \x20\x20\x20\x20[Content]: {}\n\
                                 \x20\x20</MessageEntry>\n\n\
                                 - <SystemContext trigger=\"SELF_CORRECTION\" reason=\"Truncation detected (ends with ':')\" directive=\"Continue your response starting from the code block.\" />\n",
                                timestamp, chunk.content
                            ));
                            continue;
                        }

                        turn_text.push_str(&chunk.content);
                        accumulated_content.push_str(&chunk.content);
                    }

                    // Append model's output to history_text to ensure context persists across the loop turns
                    if !turn_text.is_empty() {
                        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
                        history_text.push_str(&format!(
                            "- <MessageEntry timestamp=\"{}\" id=\"LIVE_TURN_NOMI\" type=\"ASSISTANT_THOUGHT\">\n\
                             \x20\x20\x20\x20[Actor]: Nomi\n\
                             \x20\x20\x20\x20[Content]: {}\n\
                             \x20\x20</MessageEntry>\n\n",
                            timestamp, turn_text
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
                            history_text.push_str("- <SystemContext trigger=\"ORCHESTRATOR\" reason=\"Tool results detected with no final response\" directive=\"Please synthesize the tool results into a final response for the user. Do not call the same tools again.\" />\n");
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
                        if let Ok(new_prompt) = retrieval.generate_system_prompt(&self.current_user, &conversation, &intents, &memories_text).await {
                            system_prompt = new_prompt;
                        }
                    }
                    // --- 🚨 START METADATA INTERCEPTION HOOK 🚨 ---
                    for (name, result) in &tool_results {
                        if result.success {
                            // 1. Capture ref_id into a list for global traceability
                            if !result.ref_id.is_empty() {
                                if let Some(acc_obj) = accumulated_metadata.as_object_mut() {
                                    let refs = acc_obj.entry("tool_ref_ids").or_insert(json!([]));
                                    if let Some(refs_arr) = refs.as_array_mut() {
                                        refs_arr.push(json!({
                                            "tool": name,
                                            "ref_id": result.ref_id
                                        }));
                                    }
                                }
                            }

                            // 2. Legacy string-based metadata capture [METADATA: ...]
                            if let Some(start_idx) = result.content.find("[METADATA:") {
                                if let Some(end_idx) = result.content[start_idx..].find(']') {
                                    let meta_raw = &result.content[start_idx + 10..start_idx + end_idx];
                                    if let Ok(meta_json) = serde_json::from_str::<serde_json::Value>(meta_raw) {
                                        // Merge new metadata into accumulated_metadata
                                        if let Some(acc_obj) = accumulated_metadata.as_object_mut() {
                                            if let Some(new_obj) = meta_json.as_object() {
                                                for (k, v) in new_obj {
                                                    acc_obj.insert(k.clone(), v.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // --- 🚨 END METADATA INTERCEPTION HOOK 🚨 ---

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
                        if let Ok(new_prompt) = retrieval.generate_system_prompt(&self.current_user, &conversation, &intents, &memories_text).await {
                            system_prompt = new_prompt;
                        }
                        loop_count -= 1; // Retry this iteration
                        continue;
                    }

                    // 🌟 UNIFIED TELEMETRY FORMATTING: Using the centralized HighFidelityHistory utility
                    let telemetry = crate::feature::message_processor::history_utils::HighFidelityHistory::format_tool_results(
                        &tool_results
                    );
                    history_text.push_str(&telemetry);

                    tool_turns.push((current_calls, tool_results.clone()));

                    // --- 🚨 SELF-REINFORCEMENT ENGINE TRIGGER 🚨 ---
                    let pool_clone = state.pool.clone();
                    let gemini_clone = state.gemini.clone();
                    let raw_user_text = msg.text_content.clone();
                    let dispatcher_clone = dispatcher.clone();

                    let executed_slugs: Vec<(String, String)> = tool_results.iter()
                        .filter_map(|(name, res)| {
                            if res.success {
                                // Extract description from either static or dynamic plugins
                                let description = if let Some(static_plugin) = dispatcher_clone.plugins.get(name.as_str()) {
                                    static_plugin.schema()["description"].as_str().unwrap_or_default().to_string()
                                } else {
                                    // Fallback for dynamic plugins - we'll let the reinforcement service handle the lookup if needed
                                    // but we pass name as slug for now.
                                    String::new()
                                };
                                Some((name.clone(), description))
                            } else {
                                None
                            }
                        })
                        .collect();

                    tokio::spawn(async move {
                        for (slug, base_desc) in executed_slugs {
                            if let Err(e) = crate::services::static_reinforcement::reinforce_static_plugin_profile(
                                pool_clone.clone(), gemini_clone.clone(), slug, raw_user_text.clone(), base_desc
                            ).await {
                                error!("Failed plugin reinforcement pass: {}", e);
                            }
                        }
                    });
                    // --- 🚨 END SELF-REINFORCEMENT ENGINE TRIGGER 🚨 ---
                }
                Err(e) => {
                    error!("V2 Agentic loop error: {}", e);

                    let err_str = e.to_string();
                    let custom_error_msg = if err_str.contains("429") || err_str.contains("spending cap") || err_str.contains("RESOURCE_EXHAUSTED") {
                        "⚠️ *GEMINI API SPENDING LIMIT EXCEEDED (429)*\n\nOops! It looks like our Gemini API key has exceeded its monthly spending cap or hit a rate limit.\n\nPlease visit AI Studio at https://ai.studio/spend to check your billing and manage your project's spend cap. Once updated, I'll be ready to pick right back up! 🚀".to_string()
                    } else {
                        format!("⚠️ *AGENT TURN ERROR*\n\nOops, I ran into a system error while processing your request: `{}`\n\nPlease try again or check the system logs.", err_str)
                    };

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
                        custom_error_msg.clone(),
                    )
                    .await;

                    // Also save the message to the conversation history so the user sees it immediately in their thread!
                    let save_res = save_message(
                        &state.pool,
                        conversation_id,
                        "assistant",
                        &custom_error_msg,
                        None,
                        None,
                        0, 0, 0,
                        None, None, None, None, None, None, None,
                        Some(&state.redis)
                    ).await;

                    if let Ok(saved_msg) = save_res {
                        let member_ids = self.conversation_member_ids.iter().map(|v| v.clone()).collect();
                        let _ = send_message_to_subscriber(
                            &state,
                            member_ids,
                            conversation_id,
                            msg.source.clone(),
                            saved_msg.to_sse_json(0),
                            saved_msg.into()
                        ).await;
                    }

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
                total_tokens + intent.total_tokens.to_i32().unwrap_or(0),
                None,
                None,
                None,
                None,
                None,
                Some(accumulated_metadata.clone()),
                None,
                Some(&state.redis),
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
                    record.clone().into(),
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

        let retrieval = crate::rag::RagRetrieval::new(self.state.clone(), dispatcher.clone());

        let intents_list = vec!["FULL_REGISTRY".to_string()];
        let system_prompt = retrieval
            .generate_system_task_prompt(&workspace_bootstrap, &workspace_soul, &trigger, &intents_list)
            .await?;

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
        let mut accumulated_metadata = json!({});

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

                    // --- 🚨 START METADATA INTERCEPTION HOOK 🚨 ---
                    for (name, result) in &tool_results {
                        if result.success {
                            // 1. Capture ref_id into a list for global traceability
                            if !result.ref_id.is_empty() {
                                if let Some(acc_obj) = accumulated_metadata.as_object_mut() {
                                    let refs = acc_obj.entry("tool_ref_ids").or_insert(json!([]));
                                    if let Some(refs_arr) = refs.as_array_mut() {
                                        refs_arr.push(json!({
                                            "tool": name,
                                            "ref_id": result.ref_id
                                        }));
                                    }
                                }
                            }
                        }
                    }
                    // --- 🚨 END METADATA INTERCEPTION HOOK 🚨 ---

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
            Some(accumulated_metadata.clone()),
            None,
            Some(&self.state.redis),
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
                msg.into()
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
