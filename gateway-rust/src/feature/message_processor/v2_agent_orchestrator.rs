use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::classification::{
    classification, classify_intent, fetch_media_from_storage,
};
use crate::common::agent::execute_tools;
use crate::common::identity::UserIdentity;
use crate::common::repository::message_repo::save_message;
use crate::common::tools::ToolDispatcher;
use crate::feature::message_processor::v2_orchestrator::{
    send_message_to_subscriber, send_status_presence_update, send_status_update,
};
use crate::feature::{MessageSource, UnifiedMessage};
use crate::models::Conversation;
use crate::rag::trigger_memory_consolidation;
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
            msg.sticker_url.clone(),
        )
        .await;
        if let Err(e) = save_user_message {
            info!("Saving message failed :{}", e);
            return Ok(());
        }

        let mut saved_message = save_user_message?;

        //notify message incoming
        for member in self.conversation_member_ids.iter() {
            info!("notify user message saved :{}", member);
            saved_message.display_name = Some(msg.display_name.clone().unwrap());
            let _ = state
                .send_sse_to_user(
                    member.to_string().as_str(),
                    "message",
                    saved_message.to_sse_json(0),
                )
                .await;
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
                .broadcast_sse_token_update(
                    &conversation_id,
                    &row.cumulative_tokens.unwrap_or(0).to_u64().unwrap_or(0),
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

        let (augmented_text, _media_context, should_ignore) = classification(
            &state,
            self.conversation_member_ids
                .iter()
                .map(|v| v.clone())
                .collect(),
            conversation_id,
            &msg,
            text_content.clone(),
            injected_system_prompt,
        )
        .await;

        if should_ignore {
            info!("Media classification returned IGNORE, stopping orchestrator loop");
            return Ok(());
        }

        // Fetch media data if present for Multi-Part prompt
        let media_data = if let Some(ref url) = msg.image_url {
            fetch_media_from_storage(&state, url).await.ok()
        } else {
            None
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
                Some(path) if !is_processed => {
                    format!(" - Image: {} \n", state.storage.get_full_url(&path))
                }
                _ => "".to_string(),
            };
            let video_url = match msg_h.video_url {
                Some(path) if !is_processed => {
                    format!("- Video: {} \n", state.storage.get_full_url(&path))
                }
                _ => "".to_string(),
            };
            let audio_url = match msg_h.audio_url {
                Some(path) if !is_processed => {
                    format!(" - Audio: {} \n", state.storage.get_full_url(&path))
                }
                _ => "".to_string(),
            };
            let document_url = match msg_h.document_url {
                Some(path) if !is_processed => {
                    format!("- Document: {} \n", state.storage.get_full_url(&path))
                }
                _ => "".to_string(),
            };

            let sticker_url = match msg_h.sticker_url {
                Some(path) if !is_processed => {
                    format!("- Sticker: {} \n", state.storage.get_full_url(&path))
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
                "-[{}] {}: {}.\n {}{}{}{}{}",
                msg_h
                    .created_at
                    .unwrap_or(Utc::now())
                    .format("%Y-%m-%d %H:%M")
                    .to_string(),
                role_label,
                msg_h.content,
                image_url,
                video_url,
                audio_url,
                document_url,
                sticker_url
            ));
        }

        let mut intents =
            classify_intent(state.gemini.as_ref(), &augmented_text, &history_text).await;

        // Fallback Logic: override GENERAL if imperative verbs or URLs are present
        let msg_lower = augmented_text.to_lowercase();
        if intents.contains(&"GENERAL".to_string()) && intents.len() == 1 {
            if msg_lower.contains("http://")
                || msg_lower.contains("https://")
                || msg_lower.contains("check")
                || msg_lower.contains("log")
                || msg_lower.contains("find")
                || msg_lower.contains("search")
                || msg_lower.contains("save")
                || msg_lower.contains("tell")
                || msg_lower.contains("message")
            {
                intents = vec!["FULL_REGISTRY".to_string()];
                info!("Scout overridden: Fallback to FULL_REGISTRY due to keywords");
            }
        }

        // Force FINANCE if money/spending keywords are present
        if msg_lower.contains("spend")
            || msg_lower.contains("expense")
            || msg_lower.contains("money")
            || msg_lower.contains("receipt")
            || msg_lower.contains("finance")
            || msg_lower.contains("transaction")
        {
            if !intents.contains(&"FINANCE".to_string()) {
                intents.push("FINANCE".to_string());
                info!("Scout overridden: Added FINANCE intent due to keywords");
            }
        }

        // Force VITALITY if health keywords are present
        if msg_lower.contains("step")
            || msg_lower.contains("sleep")
            || msg_lower.contains("heart")
            || msg_lower.contains("workout")
            || msg_lower.contains("health")
        {
            if !intents.contains(&"VITALITY".to_string()) {
                intents.push("VITALITY".to_string());
                info!("Scout overridden: Added VITALITY due to health keywords");
            }
        }

        info!("Scout Intents Detected: {:?}", intents);

        let dispatcher = ToolDispatcher::new(
            state.pool.clone(),
            std::env::current_dir().unwrap_or_default(),
            user_id.clone(),
            Some(conversation_id),
            state.gemini.clone(),
            state.gemini_api_key.clone(),
            state.sse.clone(),
            state.storage.clone(),
            state.clone(),
        );

        if let None = self.conversation {
            info!("conversation is null {:?}", self.conversation);
            return Ok(());
        }

        let conversation = self.conversation.clone().unwrap();

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

            if !intents_val.contains(&"GENERAL".to_string()) || intents_val.len() > 1 {
                let domain_rules = crate::prompts::PromptRegistry::domain_logic(intents_val);
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
        let mut has_retried = false;

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
                let _ = send_status_update(
                    &state,
                    self.conversation_member_ids
                        .iter()
                        .map(|v| v.clone())
                        .collect(),
                    conversation_id,
                    MessageSource::Web {name:"web".to_string()},
                    msg.is_group,
                    "thought".to_string(),
                    crate::prompts::StatusRegistry::random_thinking_phrase(),
                )
                .await;
            }

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
                            None => state.broadcast_sse("thought", payload).await,
                            Some(ref id) => {
                                state
                                    .send_sse_to_user(id.to_string().as_str(), "thought", payload)
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

                    // Status: Tool checking
                    for call in &current_calls {
                        let _ = send_status_update(
                            &state,
                            self.conversation_member_ids
                                .iter()
                                .map(|v| v.clone())
                                .collect(),
                            conversation_id,
                            MessageSource::Web {name:"web".to_string()},
                            msg.is_group,
                            "tool_start".to_string(),
                            crate::prompts::StatusRegistry::random_action_phrase(&call.name),
                        )
                        .await;
                    }

                    let tool_results = execute_tools(
                        &dispatcher,
                        current_calls.clone(),
                        &text_content, // use the v2-stripped one
                        Some(state.sse.clone()),
                    )
                    .await;

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


                let _ = send_message_to_subscriber(
                    &state,
                    self.conversation_member_ids
                        .iter()
                        .map(|v| v.clone())
                        .collect(),
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
                let _ = trigger_memory_consolidation(
                    pool,
                    gemini,
                    gemini_api_key,
                    conversation_id,
                    sse,
                )
                .await;
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
            self.state.sse.clone(),
            self.state.storage.clone(),
            self.state.clone(),
        );

        let build_system_prompt = || -> String {
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
            combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());

            combined
        };

        let system_prompt = build_system_prompt();
        let intents = vec!["FULL_REGISTRY".to_string()];

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
                crate::common::agent::send_prompt(&dispatcher, current_actor, &intents).await;

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
                        execute_tools(&dispatcher, current_calls.clone(), task_prompt, None).await;

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
                json!({}),
                msg,
            )
            .await;
        }

        Ok(final_text)
    }
}

fn strip_thinking_tags(text: &str) -> String {
    let healed = crate::common::format::heal_thinking_tags(text);
    let re = regex::Regex::new(r"(?si)<thinking>.*?</thinking>|<thinking>.*").unwrap();
    re.replace_all(&healed, "").trim().to_string()
}
