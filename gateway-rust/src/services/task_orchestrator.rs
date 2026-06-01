use crate::common::app_state::AppState;
use crate::common::tools::ToolDispatcher;
use crate::common::repository::message_repo::save_message;
use crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber;
use crate::feature::MessageSource;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tracing::{error, info};
use uuid::Uuid;

/// Matches any natural-language or JSON signal that the *current step* is done.
fn step_done_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r##"(?i)(status\s*[=:]\s*["']?completed["']?|step\s+(is\s+|has\s+been\s+)?completed|checkpoint\s+completed|all\s+(steps|checkpoints)\s+completed|task\s+is\s+(complete|done)|successfully\s+completed|have\s+completed|has\s+been\s+fulfilled|goal\s+fulfilled|goal\s+(has\s+been\s+)?achieved|completed\s+(the\s+)?(task|final\s+goal))"##
        )
        .expect("step_done_regex is valid")
    })
}

/// Matches signals that the *entire task* (all steps) is finished.
fn final_goal_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r##"(?i)(completed\s+final\s+goal|task\s+completed|all\s+(steps|checkpoints)\s+completed|goal\s+(has\s+been\s+)?achieved|goal\s+fulfilled)"##
        )
        .expect("final_goal_regex is valid")
    })
}


#[derive(sqlx::FromRow)]
struct TaskInfo {
    conversation_id: Uuid,
    title: String,
    global_goal: String,
    checkpoints: Value,
    status: String,
    current_step_index: i32,
    sub_conversation_id: Option<Uuid>,
    soul_content: Option<String>,
    bootstrap_content: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct TimelineLog {
    step_index: i32,
    event_type: String,
    log_content: String,
    raw_payload: Value,
}

#[derive(sqlx::FromRow)]
struct OwnerInfo {
    id: Uuid,
    display_name: Option<String>,
    email: Option<String>,
}

pub fn spawn_task_loop(state: AppState, task_id: Uuid) {
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = advanced_orchestrate_task_step(state_clone, task_id).await {
            error!("Error in autonomous task orchestrator loop [{}]: {}", task_id, e);
        }
    });
}

pub async fn advanced_orchestrate_task_step(
    state: AppState,
    task_id: Uuid,
) -> anyhow::Result<()> {
    info!("🚀 Initializing Autonomous Task Worker loop for ID: {}", task_id);
    let pool = state.pool.clone();
    
    // Safety counter to prevent infinite execution loops in background
    let mut iteration_limit = 20;
    // Counter for turns where no progress was made (no tools, no suspension, no step_completed)
    let mut stuck_turn_count = 0;

    while iteration_limit > 0 {
        iteration_limit -= 1;

        // 1. Fetch live task coordinates along with room configuration and active persona using non-macro query_as
        let task_row = sqlx::query_as::<_, TaskInfo>(
            "SELECT t.conversation_id, t.title, t.global_goal, t.checkpoints, t.status, t.current_step_index, \
                    t.sub_conversation_id, c.soul_content, c.bootstrap_content \
             FROM autonomous_tasks t \
             JOIN conversations c ON t.conversation_id = c.id \
             WHERE t.id = $1 LIMIT 1"
        )
        .bind(task_id)
        .fetch_optional(&pool)
        .await?;

        let Some(task) = task_row else {
            error!("Task ID [{}] not found in ledger database.", task_id);
            return Ok(());
        };

        // If the task status isn't running (e.g. paused_for_input, completed, failed), exit loop
        if task.status != "running" {
            info!("Task [{}] is no longer running (current status: '{}'). Gracefully exiting thread.", task_id, task.status);
            return Ok(());
        }

        let conversation_id = task.conversation_id;
        let global_goal = task.global_goal;
        let current_step_index = task.current_step_index;
        let checkpoints = task.checkpoints;
        let mut sub_conversation_id = task.sub_conversation_id;

        // 4. Structured User Profile & RAG Memory Scan (Proactive Data Retrieval) using non-macro query_as
        let mut owner_row = sqlx::query_as::<_, OwnerInfo>(
            "SELECT u.id, u.display_name, u.email FROM users u \
             JOIN conversations c ON c.user_id = u.id \
             WHERE c.id = $1 LIMIT 1"
        )
        .bind(conversation_id)
        .fetch_optional(&pool)
        .await?;

        // Fallback to the primary user in the system if the conversation is not directly linked (e.g. dynamic sub-chats)
        if owner_row.is_none() {
            info!("HTO: Conversation {} not linked to a user. Falling back to primary system user.", conversation_id);
            owner_row = sqlx::query_as::<_, OwnerInfo>(
                "SELECT id, display_name, email FROM users ORDER BY created_at ASC LIMIT 1"
            )
            .fetch_optional(&pool)
            .await?;
        }

        let user_display_name = owner_row.as_ref().map(|o| o.display_name.clone().unwrap_or_default()).unwrap_or_else(|| "Trian".to_string());
        let user_email = owner_row.as_ref().map(|o| o.email.clone().unwrap_or_default()).unwrap_or_default();

        // Build the dynamic ToolDispatcher to execute Nomi tools in the background context
        let dispatcher = ToolDispatcher::new(
            pool.clone(),
            std::path::PathBuf::from("."),
            owner_row.as_ref().map(|o| o.id), // Use the actual authentic owner user ID!
            Some(conversation_id),
            state.gemini.clone(),
            state.gemini_api_key.clone(),
            state.storage.clone(),
            state.clone(),
        );

        // Instantiate the universal RagRetrieval context builder
        let retrieval = crate::rag::RagRetrieval::new(state.clone(), dispatcher.clone())
            .with_history(15)
            .with_simple_history(true);

        // 2. Fetch Twin chronological conversation history contexts (Step 3)
        // A. Fetch last 15 messages from Parent Conversation (Owner) via universal retrieval
        let parent_history = retrieval.fetch_history().await?;

        // B. Fetch last 15 messages from Sub-Conversation (Target) if exists via universal retrieval
        let sub_history = if let Some(sub_convo_id) = sub_conversation_id {
            let sub_dispatcher = ToolDispatcher::new(
                pool.clone(),
                std::path::PathBuf::from("."),
                owner_row.as_ref().map(|o| o.id),
                Some(sub_convo_id),
                state.gemini.clone(),
                state.gemini_api_key.clone(),
                state.storage.clone(),
                state.clone(),
            );

            let sub_retrieval = crate::rag::RagRetrieval::new(state.clone(), sub_dispatcher)
                .with_history(15)
                .with_simple_history(true);

            sub_retrieval.fetch_history().await.unwrap_or_default()
        } else {
            "No active sub-conversation mapped yet.".to_string()
        };

        // 3. Contextual Timeline Log Rehydration (Scratchpad Memory) using non-macro query_as
        let previous_logs = sqlx::query_as::<_, TimelineLog>(
            "SELECT step_index, event_type, log_content, raw_payload FROM autonomous_task_logs \
             WHERE task_id = $1 ORDER BY created_at ASC"
        )
        .bind(task_id)
        .fetch_all(&pool)
        .await?;

        let mut scratchpad = String::new();
        for log in &previous_logs {
            scratchpad.push_str(&format!(
                "[Step {}] EVENT: '{}' | Details: {}\n",
                log.step_index, log.event_type, log.log_content
            ));
        }

        // Perform RAG vector lookup for user context preferences via universal retrieval
        let mut rag_context = String::new();
        let memories_text = retrieval.clone()
            .with_retrieval(global_goal.clone())
            .fetch_memories()
            .await
            .unwrap_or_default();

        if !memories_text.is_empty() {
            for record in memories_text.split("---") {
                rag_context.push_str(&format!("* {}\n", record));
            }
        }

        // 5. Build HTO Operational System Prompt
        let soul_text = task.soul_content.clone().unwrap_or_else(|| "You are Nomi, a helpful AI teammate.".to_string());
        let bootstrap_text = task.bootstrap_content.unwrap_or_default();

        let hto_prompt = format!(
            "Persona Core Prompt:\n{}\n\nBootstrap Prompt:\n{}\n\n\
             ===================================================\n\
             AUTONOMOUS PLANNER ROLE:\n\
             You are operating in HTO (Hierarchical Task Orchestrator) Autonomous Mode. You are executing an asynchronous, multi-step background pipeline to fulfill a target user goal.\n\n\
             USER INFO CONTEXT:\n\
             - User Primary Name: \"{}\"\n\
             - User Email: \"{}\"\n\
             - Inferred Preferences from Memory (RAG):\n{}\n\n\
             ===================================================\n\
             TWIN-CHANNEL CHAT HISTORY CONTEXT:\n\n\
             [A] OWNER CONVERSATION HISTORY (Parent Chat):\n\
             {}\n\n\
             [B] TARGET CONVERSATION HISTORY (Sub-Chat with Target Person):\n\
             {}\n\n\
             ===================================================\n\
             TASK PARAMETERS:\n\
             - Global Task Goal: \"{}\"\n\
             - Current Step Index: {}\n\
             - Complete Checkpoints Plan (JSONB):\n{}\n\n\
             CHRONOLOGICAL TASK TIMELINE SCRATCHPAD:\n\
             {}\n\n\
             ===================================================\n\
             INSTRUCTIONS & LOGIC FLOW:\n\
             1. Evaluate the completed logs on the scratchpad carefully to understand exactly what you have done and what data was resolved.\n\
             2. Determine what action is required next to satisfy the goal of step index {}.\n\
             3. You have full access to Nomi's plugin toolset. If you need to perform an action (e.g. check stock, search web, manage finance, send message, retrieve knowledge), you MUST respond with a structured tool call. If you don't know the exact tool, invoke 'discover_tools(query)'.\n\
             4. SUSPENSION RULE A: If you need dynamic input or clarification from the OWNER (Trian) to proceed with your workflow (e.g. confirming choices, or asking questions), transition the task status by outputting a status update with status=\"paused_for_input\" and detail the questions. This suspends your thread until the Owner replies.\n\
             5. SUSPENSION RULE B: Once you have successfully called `send_message` to the target external person, you MUST IMMEDIATELY transition the task status by outputting a status update with status=\"waiting_external_feedback\" in the exact same turn or the very next turn. This suspends your thread until they reply. You MUST NOT call `schedule_task` or other tools to wait or check in; the system event bus will automatically wake you up when their reply arrives.\n\
             6. When the current step index is completed, output a JSON structure explaining the progress updates to update checkpoints.\n\
             7. If ALL steps in the HTO plan are fully completed and the global goal is satisfied, mark the final task status as \"completed\".\n\
             8. CANCELLATION RULE: If the OWNER (Trian) explicitly asks you in their input message to \"cancel\", \"stop\", \"abort\", or \"cancel dulu\" the task, you MUST IMMEDIATELY transition the task status by outputting a status update with status=\"failed\" and explain in natural language that you have cancelled the task per their request. You MUST NOT execute any more tools or perform any more actions. This will stop the task execution cleanly.\n\
             9. MANDATORY RULE: Never guess, hallucinate, or pass phone numbers, handles, JIDs or numeric strings (like '@2297908166856') to the 'user_id' parameter of `send_message`. You MUST always invoke the search tool (`manage_user` with action='search') first to find the user's database UUID, then pass that UUID directly into the 'user_id' parameter of `send_message`. Any target parameter that is not a valid 36-character UUID will fail instantly.\n\
             10. SCHEDULING RULE: If you invoke the `schedule_task` tool, you MUST pass the `due_at` parameter in the exact format 'YYYY-MM-DD HH:MM'. You MUST NOT pass text like 'tonight', 'tomorrow', 'now', or '3 hours' as these are invalid and will cause immediate tool execution failures. If you don't know the exact date/time, calculate it based on the WIB system time anchor provided in the tool results or ask the OWNER for clarification.\n\
             11. CONTINUOUS EXECUTION PROTOCOL (CRITICAL): You are executing inside an automatic, continuous background loop on a single dedicated Tokio thread. Consecutive steps in your checkpoints plan are executed sequentially in subsequent turns of this active loop.\n\
                 - DO NOT try to schedule or spawn a new background task (such as AUTONOMOUS_TASK or TRIGGER_AGENT) via the `schedule_task` tool to progress to the next checkpoint or step! You are forbidden from scheduling recursive background tasks from inside an existing background task.\n\
                 - To transition to the next step index, simply output a JSON block updating the checkpoints/status (with 'completed': true or 'status': 'completed' for the current step) WITHOUT executing any further tools in that same turn.\n\
                 - The HTO loop engine will automatically detect your JSON checkpoint update, save it to the database, increment your step index, and immediately invoke you again in the next turn of the current active thread.\n\n\
             Respond in a highly structured manner.",
            soul_text,
            bootstrap_text,
            user_display_name,
            user_email,
            rag_context,
            parent_history,
            sub_history,
            global_goal,
            current_step_index,
            checkpoints,
            scratchpad,
            current_step_index
        );

        let send_res = crate::common::agent::send_prompt(
            &dispatcher,
            crate::common::agent::agent_model::PromptActor::User {
                history: "".to_string(),
                memories: "".to_string(),
                message: "Evaluate scratchpad, execute the next tool, or update checkpoints plan.".to_string(),
                system_prompt: hto_prompt,
                media: None,
            },
            &["HTO_WORKFLOW_REGISTRY".to_string()],
        )
        .await;

        let (response, _) = match send_res {
            Ok(val) => val,
            Err(e) => {
                let err_str = e.to_string();
                error!("HTO Loop Error for task [{}]: {}", task_id, err_str);

                let custom_error_msg = if err_str.contains("429") || err_str.contains("spending cap") || err_str.contains("RESOURCE_EXHAUSTED") {
                    "⚠️ *GEMINI API SPENDING LIMIT EXCEEDED (429)*\n\nOops! It looks like our Gemini API key has exceeded its monthly spending cap or hit a rate limit.\n\nPlease visit AI Studio at https://ai.studio/spend to check your billing and manage your project's spend cap. Once updated, I'll be ready to pick right back up! 🚀".to_string()
                } else {
                    format!("⚠️ *HTO WORKFLOW ERROR*\n\nOops, I ran into a system error while executing the background workflow step: `{}`", err_str)
                };

                // Save message to conversation
                let save_res = save_message(
                    &pool,
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
                    let members: Vec<Uuid> = sqlx::query_scalar(
                        "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
                    )
                    .bind(conversation_id)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();

                    send_message_to_subscriber(
                        &state,
                        members,
                        conversation_id,
                        MessageSource::Web { name: "web".to_string() },
                        saved_msg.to_sse_json(0),
                        saved_msg.into()
                    )
                    .await;
                }

                // Update task status in DB to 'failed'
                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET status = 'failed' WHERE id = $1"
                )
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                return Err(anyhow::anyhow!("HTO Loop Error: {}", err_str));
            }
        };

        // Extract token usage metadata from the operational Gemini turn
        let mut input_tokens = 0;
        let mut output_tokens = 0;
        let mut total_tokens = 0;
        if let Some(usage) = &response.usage_metadata {
            input_tokens = usage.prompt_token_count.unwrap_or(0) as i64;
            output_tokens = usage.candidates_token_count.unwrap_or(0) as i64;
            total_tokens = usage.total_token_count.unwrap_or(0) as i64;
        }

        if total_tokens > 0 {
            info!("HTO: background operational turn consumed {} tokens (Input: {}, Output: {})", total_tokens, input_tokens, output_tokens);
            
            // 1. Log to unified token usage history ledger for system analytics
            let _ = sqlx::query(
                "INSERT INTO token_usage_history (conversation_id, user_id, type, role, input_tokens, output_tokens, total_tokens) \
                 VALUES ($1, $2, 'autonomous_task_hto', 'assistant', $3, $4, $5)"
            )
            .bind(conversation_id)
            .bind(dispatcher.user_id)
            .bind(input_tokens)
            .bind(output_tokens)
            .bind(total_tokens)
            .execute(&pool)
            .await;

            // 2. Increment cumulative task token counter on main task ledger (gracefully fails if column does not exist yet)
            let _ = sqlx::query(
                "UPDATE autonomous_tasks SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1 WHERE id = $2"
            )
            .bind(total_tokens)
            .bind(task_id)
            .execute(&pool)
            .await;
        }

        let raw_text = response.text().trim().to_string();
        let healed_text = crate::common::format::heal_thinking_tags(&raw_text);
        let parsed = crate::common::agent::parse_llm_output(&healed_text);
        let response_text = healed_text.clone();
        
        info!("Background LLM response (raw): {}", raw_text);
        info!("Background LLM response (healed): {}", response_text);
        info!("Extracted Cleaned Response: {}", parsed.response);
        info!("Extracted Thought: {}", parsed.thought);

        // Compute suspension flags robustly to catch single/double/no quotes and dash/underscore variations
        let lower_res = response_text.to_lowercase();
        let is_external_feedback = lower_res.contains("status=\"waiting_external_feedback\"") 
            || lower_res.contains("status='waiting_external_feedback'")
            || lower_res.contains("status=waiting_external_feedback")
            || lower_res.contains("status=\"waiting-external-feedback\"")
            || lower_res.contains("status='waiting-external-feedback'")
            || lower_res.contains("status=waiting-external-feedback")
            || lower_res.contains("waiting_external_feedback")
            || lower_res.contains("waiting-external-feedback");

        let is_paused_input = lower_res.contains("status=\"paused_for_input\"")
            || lower_res.contains("status='paused_for_input'")
            || lower_res.contains("status=paused_for_input")
            || lower_res.contains("<requiredfield")
            || lower_res.contains("status=\"paused-for-input\"")
            || lower_res.contains("status='paused-for-input'")
            || lower_res.contains("status=paused-for-input")
            || lower_res.contains("paused_for_input")
            || lower_res.contains("paused-for-input");

        // A. Handle Tool Call Detections
        let function_calls = response.function_calls();
        if !function_calls.is_empty() {
            info!("HTO: background LLM issued tool execution calls: {:?}", function_calls);

            // 1. Write 'step_start' timeline entry if not logged
            let _ = sqlx::query(
                "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                 VALUES ($1, $2, 'step_start', $3, $4)"
            )
            .bind(task_id)
            .bind(current_step_index)
            .bind(format!("Started executing step: {}", current_step_index))
            .bind(json!({}))
            .execute(&pool)
            .await;

            let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

            // Execute function calls
            let tool_results = crate::common::agent::execute_tools(
                &dispatcher,
                function_calls.iter().copied().cloned().collect(),
                json!({}),
                json!({})
            )
            .await;

            for (name, result) in tool_results {
                info!("Tool [{}] execute result: success={}, content={}", name, result.success, result.content);
                
                let log_detail = if result.success {
                    format!("Tool [{}] executed. Result success: true. Response: {}", name, result.content)
                } else {
                    format!("Tool [{}] executed. Result success: false. Error: {}. Response: {}", name, result.error, result.content)
                };

                // Write 'tool_execution' timeline event
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'tool_execution', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind(log_detail)
                .bind(json!({
                    "tool": name,
                    "success": result.success,
                    "content": result.content,
                    "ref_id": result.ref_id
                }))
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                // Dynamic isolate sub-conversation mapping (Phase 2 WhatsApp/Telegram sub-chats)
                if name == "send_message" && result.success {
                    // Extract true JID target from JID-encoded ref_id
                    let mut target_jid = "".to_string();
                    if result.ref_id.starts_with("JID:") {
                        if let Some(pipe_idx) = result.ref_id.find('|') {
                            target_jid = result.ref_id["JID:".len()..pipe_idx].to_string();
                        }
                    }

                    // Fallback to old arguments extraction
                    if target_jid.is_empty() {
                        for call in &function_calls {
                            if call.name == "send_message" {
                                if let Some(target_val) = call.args.get("target") {
                                    if let Some(target_str) = target_val.as_str() {
                                        target_jid = target_str.to_string();
                                    }
                                } else if let Some(uid_val) = call.args.get("user_id") {
                                    if let Some(uid_str) = uid_val.as_str() {
                                        target_jid = uid_str.to_string();
                                    }
                                }
                            }
                        }
                    }

                    if !target_jid.is_empty() && sub_conversation_id.is_none() {
                        info!("Dynamic isolation: spawning a new sub-chat room for JID: {}", target_jid);
                        
                        let mut target_user_id: Option<Uuid> = None;
                        
                        // Try to find the target user ID from send_message arguments
                        for call in &function_calls {
                            if call.name == "send_message" {
                                if let Some(uid_val) = call.args.get("user_id") {
                                    if let Some(uid_str) = uid_val.as_str() {
                                        if let Ok(parsed) = Uuid::parse_str(uid_str) {
                                            target_user_id = Some(parsed);
                                        }
                                    }
                                }
                            }
                        }

                        // If not found in arguments, look up via target_jid in the users table
                        if target_user_id.is_none() {
                            if let Ok(opt_id) = sqlx::query_scalar::<_, Uuid>(
                                "SELECT id FROM users WHERE external_id = $1"
                            )
                            .bind(&target_jid)
                            .fetch_optional(&pool)
                            .await {
                                target_user_id = opt_id;
                            }
                        }
                        
                        // Spawn isolated conversation row using non-macro query_scalar (tie to parent_id and user_id/target)
                        let sub_title = format!("{} (Sub-chat for Task)", task.title);
                        let sub_convo_id = sqlx::query_scalar::<_, Uuid>(
                            "INSERT INTO conversations (title, conversation_type, soul_content, bootstrap_content, parent_id, user_id) \
                             VALUES ($1, 'channel_subchat', $2, $3, $4, $5) RETURNING id"
                        )
                        .bind(sub_title)
                        .bind(soul_text.clone())
                        .bind(bootstrap_text.clone())
                        .bind(conversation_id) // parent conversation ID
                        .bind(target_user_id)  // target user database UUID
                        .fetch_one(&pool)
                        .await?;

                        // Insert target user to conversation members of the sub-conversation
                        if let Some(target_uid) = target_user_id {
                            let _ = sqlx::query(
                                "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
                            )
                            .bind(sub_convo_id)
                            .bind(target_uid)
                            .execute(&pool)
                            .await;
                        }

                        // Insert actor (owner/initiator) to conversation members of the sub-conversation
                        if let Some(actor_uid) = dispatcher.user_id {
                            let _ = sqlx::query(
                                "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
                            )
                            .bind(sub_convo_id)
                            .bind(actor_uid)
                            .execute(&pool)
                            .await;
                        }

                        // Insert channel mapping using non-macro query
                        let _ = sqlx::query(
                            "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id) \
                             VALUES ('whatsapp', $1, $1, $2)"
                        )
                        .bind(target_jid)
                        .bind(sub_convo_id)
                        .execute(&pool)
                        .await;

                        // Save pointer to main task ledger using non-macro query
                        let _ = sqlx::query(
                            "UPDATE autonomous_tasks SET sub_conversation_id = $1 WHERE id = $2"
                        )
                        .bind(sub_convo_id)
                        .bind(task_id)
                        .execute(&pool)
                        .await;

                        sub_conversation_id = Some(sub_convo_id);
                    }
                }
            }

            // 5. Resilience Guard: Count continuous step tool run failures to prevent infinite loops
            let failed_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM autonomous_task_logs \
                 WHERE task_id = $1 AND step_index = $2 AND event_type = 'tool_execution' AND (raw_payload->>'success') = 'false'"
            )
            .bind(task_id)
            .bind(current_step_index)
            .fetch_one(&pool)
            .await
            .unwrap_or(0);

            if failed_count >= 5 {
                info!("HTO: Step {} failed 5 times continuously. Cancelling task.", current_step_index);

                // Write 'system_error' timeline audit log
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'system_error', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind("Task step execution failed continuously 5 times. Cancelling task to save resources.")
                .bind(json!({ "reason": "Max retries reached (5 failed tool execution attempts)" }))
                .execute(&pool)
                .await;

                // Update task status in DB to 'failed'
                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET status = 'failed' WHERE id = $1"
                )
                .bind(task_id)
                .execute(&pool)
                .await;

                // Stream real-time failure state via MQTT EventBus
                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                // Send teammate-style error/cancellation explanation to the chat room
                let soul_content = task.soul_content.clone().unwrap_or_else(|| "You are Nomi, a helpful AI teammate.".to_string());
                let error_prompt = format!(
                    "Your persona/soul instructions:\n=== START PERSONA ===\n{}\n=== END PERSONA ===\n\n\
                     Inform the user that the background workflow '{}' has failed or was cancelled because step {} encountered too many execution errors (failed 5 times). \
                     Explain that you stopped it to save resources, and suggest they check lookup details or parameters.\n\
                     Guidelines:\n\
                     1. Output STRLCTLY the direct casual teammate response. Do NOT prefix with any planning tags, headers, bullet points, 'Action', 'Status', or thoughts.\n\
                     2. Adopt the exact tone, style, and language guidelines defined in your soul instructions above.",
                    soul_content, task.title, current_step_index
                );

                if let Ok(err_res) = state.gemini.generate_content().with_user_message(error_prompt).with_temperature(0.7).execute().await {
                    let raw_err_msg = err_res.text().trim().to_string();
                    let healed_err = crate::common::format::heal_thinking_tags(&raw_err_msg);
                    let parsed_err = crate::common::agent::parse_llm_output(&healed_err);
                    let err_msg = parsed_err.response.trim().to_string();
                    
                    let saved = save_message(
                        &pool,
                        conversation_id,
                        "assistant",
                        &err_msg,
                        None, None, 0, 0, 0, None, None, None, None, None, None, None,
                        Some(&state.redis)
                    )
                    .await?;

                    let members: Vec<Uuid> = sqlx::query_scalar(
                        "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
                    )
                    .bind(conversation_id)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();

                    send_message_to_subscriber(
                        &state,
                        members,
                        conversation_id,
                        MessageSource::Web { name: "web".to_string() },
                        saved.to_sse_json(0),
                        saved.into()
                    )
                    .await;
                }

                return Ok(());
            }

            // If the LLM requested a suspension/pause in this turn, do NOT continue to evaluate tools; fall through to suspension block
            if is_external_feedback || is_paused_input {
                info!("HTO: Tool executed in this turn, but LLM requested suspension. Breaking tool block to suspend.");
            } else {
                // Continue the loop to evaluate the tool outcomes immediately
                continue;
            }
        }

        // B. Handle Interactive Input Pauses / Clarifications / External Feedback Pauses
        if is_external_feedback || is_paused_input {
            let target_status = if is_external_feedback { "waiting_external_feedback" } else { "paused_for_input" };
            info!("HTO loop: background planner suspends execution. Target state: '{}'.", target_status);
            
            // Extract natural message text to show Owner (Trian Damai) - strictly stripped of thoughts/thinking tags
            let cleaned_xml = parsed.response.trim().to_string();
            
            // Refine conversational text to Nomi's casual persona/slang style
            let soul_content = task.soul_content.clone().unwrap_or_else(|| "You are Nomi, a helpful AI teammate.".to_string());
            let slang_prompt = format!(
                "Your persona guidelines:\n=== START PERSONA ===\n{}\n=== END PERSONA ===\n\n\
                 You are Nomi chatting casually with Trian. Convert this raw internal planner output into a clean, brief, natural update to him: \"{}\". \
                 Guidelines:\n\
                 1. DO NOT mention raw technical terms like 'Step 0', 'Step 1', database UUIDs, JIDs, or internal tools.\n\
                 2. Just tell the user what you are doing or what you need, casually.\n\
                 3. Strictly adopt your natural persona, slang style (Indonesian/English mix, using casual words like 'aman', 'otw', 'sip', 'gua', 'lu'), and tone as defined in the persona guidelines above.\n\
                 4. Output STRICTLY the direct casual teammate response. Do NOT prefix with any planning tags, headers, bullet points, 'Action', 'Status', or thoughts.\n\
                 5. Keep the output short and highly conversational.",
                soul_content, cleaned_xml
            );
            
            let slang_res = state.gemini.generate_content()
                .with_user_message(slang_prompt)
                .with_temperature(0.7)
                .execute()
                .await?;
                
            let natural_text = slang_res.text().trim().to_string();

            // Save telemetry message (only the friendly, natural text is saved to the chat room)
            let saved = save_message(
                &pool,
                conversation_id,
                "assistant",
                &natural_text,
                None, // thought
                None, // user_id
                0, 0, 0, // tokens
                None, None, None, None, None, // media
                None, // metadata
                None, // reply_to_id
                Some(&state.redis)
            )
            .await?;

            // Broadcast to SSE & MQTT
            let members: Vec<Uuid> = sqlx::query_scalar(
                "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
            )
            .bind(conversation_id)
            .fetch_all(&pool)
            .await
            .unwrap_or_default();

            send_message_to_subscriber(
                &state,
                members,
                conversation_id,
                MessageSource::Web { name: "web".to_string() },
                saved.to_sse_json(0),
                saved.into()
            )
            .await;

            // Log timeline event
            let log_msg = if is_external_feedback {
                format!("Task suspended waiting for external feedback: {}", natural_text)
            } else {
                format!("Task paused for input: {}", natural_text)
            };

            let _ = sqlx::query(
                "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                 VALUES ($1, $2, 'outbound_msg', $3, $4)"
            )
            .bind(task_id)
            .bind(current_step_index)
            .bind(log_msg)
            .bind(json!({ "raw_xml": cleaned_xml }))
            .execute(&pool)
            .await;

            // Set task status in DB
            let _ = sqlx::query(
                "UPDATE autonomous_tasks SET status = $1 WHERE id = $2"
            )
            .bind(target_status)
            .bind(task_id)
            .execute(&pool)
            .await;

            let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

            return Ok(());
        }

        // C & D. State Machine Evaluation for Checkpoint Step Completion / Victory Progressions
        let mut step_completed = false;
        let mut is_final_goal_completed = false;
        let mut parsed_checkpoints_override = None;
        let mut next_step = current_step_index + 1;

        if let Some(parsed_json) = extract_json_structure(&response_text) {
            info!("HTO loop: parsed planning JSON: {:?}", parsed_json);
            
            // Check status indicator in JSON
            if let Some(status_str) = parsed_json.get("status").and_then(|s| s.as_str()) {
                let status_lower = status_str.to_lowercase();
                if status_lower == "completed" || status_lower == "complete" {
                    step_completed = true;
                }
            }
            
            // Or explicit boolean
            if parsed_json.get("completed").and_then(|c| c.as_bool()).unwrap_or(false) {
                step_completed = true;
            }

            // Expose checkpoints array if model generated them
            if let Some(cp_override) = parsed_json.get("checkpoints").cloned() {
                if cp_override.is_array() {
                    parsed_checkpoints_override = Some(cp_override);
                }
            }

            if let Some(next_idx) = parsed_json.get("next_step_index").and_then(|n| n.as_i64()) {
                next_step = next_idx as i32;
            }
        }

        // Regex-based fallback: catch any natural-language or JSON completion signal
        let response_lower = response_text.to_lowercase();
        if step_done_regex().is_match(&response_lower) {
            step_completed = true;
        }

        // Check if all checkpoints are done (or if the model indicates final victory)
        let mut checkpoints_arr = checkpoints.as_array().cloned().unwrap_or_default();
        let total_steps = if checkpoints_arr.is_empty() { 1 } else { checkpoints_arr.len() };

        if step_completed {
            // Automatically update our checklist status in the database to prevent manual LLM formatting errors
            let mut found = false;
            for cp in &mut checkpoints_arr {
                if let Some(idx) = cp["index"].as_i64() {
                    if idx as i32 == current_step_index {
                        if let Some(obj) = cp.as_object_mut() {
                            obj.insert("status".to_string(), json!("completed"));
                            found = true;
                        }
                    }
                }
            }

            if !found && (current_step_index as usize) < checkpoints_arr.len() {
                if let Some(obj) = checkpoints_arr[current_step_index as usize].as_object_mut() {
                    obj.insert("status".to_string(), json!("completed"));
                }
            }

            // Use the model's checkpoints array override if valid
            let final_checkpoints = parsed_checkpoints_override.unwrap_or_else(|| json!(checkpoints_arr));

            // Check if this was the last step
            // Use >= to handle both exact match and overflow; also check if current was already the last step
            if next_step as usize >= total_steps 
                || (current_step_index as usize) >= total_steps.saturating_sub(1)
                || final_goal_regex().is_match(&response_lower)
            {
                is_final_goal_completed = true;
            }

            if is_final_goal_completed {
                info!("HTO: Task loop successfully completed the global goal: '{}'", global_goal);
                
                // Log final step completion
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'step_end', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind("HTO Task global goal completed successfully.")
                .bind(json!({ "checkpoints": final_checkpoints }))
                .execute(&pool)
                .await;

                // Set final task status in DB
                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET status = 'completed', checkpoints = $1 WHERE id = $2"
                )
                .bind(final_checkpoints)
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                // Broadcast victory natural slang to user
                let soul_content = task.soul_content.clone().unwrap_or_else(|| "You are Nomi, a helpful AI teammate.".to_string());
                let victory_prompt = format!(
                    "Your persona/soul instructions:\n=== START PERSONA ===\n{}\n=== END PERSONA ===\n\n\
                     Inform the user that you have fully completed the global goal: '{}'. Celebrate this milestone!\n\
                     Guidelines:\n\
                     1. Output STRICTLY the direct casual teammate response. Do NOT prefix with any planning tags, headers, bullet points, 'Action', 'Status', or thoughts.\n\
                     2. Adopt the exact tone, style, and language guidelines defined in your soul instructions above.",
                    soul_content, global_goal
                );
                
                if let Ok(vic_res) = state.gemini.generate_content().with_user_message(victory_prompt).with_temperature(0.7).execute().await {
                    let raw_victory_msg = vic_res.text().trim().to_string();
                    let healed_victory = crate::common::format::heal_thinking_tags(&raw_victory_msg);
                    let parsed_victory = crate::common::agent::parse_llm_output(&healed_victory);
                    let victory_msg = parsed_victory.response.trim().to_string();
                    
                    let saved = save_message(
                        &pool,
                        conversation_id,
                        "assistant",
                        &victory_msg,
                        None, None, 0, 0, 0, None, None, None, None, None, None, None,
                        Some(&state.redis)
                    )
                    .await?;

                    let members: Vec<Uuid> = sqlx::query_scalar(
                        "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
                    )
                    .bind(conversation_id)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();

                    send_message_to_subscriber(
                        &state,
                        members,
                        conversation_id,
                        MessageSource::Web { name: "web".to_string() },
                        saved.to_sse_json(0),
                        saved.into()
                    )
                    .await;
                }

                return Ok(());
            } else {
                // Progressing to the next step index
                info!("HTO loop: completed step index: {}. Progressing to step: {}", current_step_index, next_step);
                
                // Log checkpoint completion
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'step_end', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind(format!("Completed step index: {}. Progressing to step: {}", current_step_index, next_step))
                .bind(json!({ "checkpoints": final_checkpoints }))
                .execute(&pool)
                .await;

                // Update task step & checkpoints in database
                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET current_step_index = $1, checkpoints = $2 WHERE id = $3"
                )
                .bind(next_step)
                .bind(final_checkpoints)
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                continue;
            }
        }

        // Catch-all: LLM gave a response with no tools, no suspension, and no completion signal.
        // This is a "stuck" turn — the model may be describing what it will do instead of acting.
        // After 3 stuck turns, force-advance to the next step to prevent silent loop exhaustion.
        stuck_turn_count += 1;
        info!("HTO: stuck turn #{} for task [{}] step {}. No tools, no suspension, no completion detected.", stuck_turn_count, task_id, current_step_index);

        if stuck_turn_count >= 3 {
            // Force-advance: mark current step completed and go to next
            info!("HTO: Force-advancing from step {} after {} stuck turns.", current_step_index, stuck_turn_count);
            stuck_turn_count = 0;

            // Mark current checkpoint as completed
            let mut force_checkpoints = checkpoints.as_array().cloned().unwrap_or_default();
            for cp in &mut force_checkpoints {
                if let Some(idx) = cp["index"].as_i64() {
                    if idx as i32 == current_step_index {
                        if let Some(obj) = cp.as_object_mut() {
                            obj.insert("status".to_string(), json!("completed"));
                        }
                    }
                }
            }
            if !force_checkpoints.is_empty() && (current_step_index as usize) < force_checkpoints.len() {
                if let Some(obj) = force_checkpoints[current_step_index as usize].as_object_mut() {
                    obj.insert("status".to_string(), json!("completed"));
                }
            }

            let force_next = current_step_index + 1;
            let force_checkpoints_val = json!(force_checkpoints);

            if force_next as usize >= total_steps {
                // This was the last step — complete the task
                info!("HTO: Force-completing task [{}] (was stuck on final step).", task_id);

                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET status = 'completed', checkpoints = $1 WHERE id = $2"
                )
                .bind(&force_checkpoints_val)
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'step_end', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind("HTO: Force-completed final step after stuck turns.")
                .bind(json!({ "checkpoints": force_checkpoints_val }))
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;

                // Send a victory message
                let soul_content = task.soul_content.clone().unwrap_or_else(|| "You are Nomi, a helpful AI teammate.".to_string());
                let victory_prompt = format!(
                    "Your persona/soul instructions:\n=== START PERSONA ===\n{}\n=== END PERSONA ===\n\n\
                     Inform the user that you have fully completed the global goal: '{}'. Celebrate this milestone!\n\
                     Guidelines:\n\
                     1. Output STRICTLY the direct casual teammate response. Do NOT prefix with any planning tags, headers, bullet points, 'Action', 'Status', or thoughts.\n\
                     2. Adopt the exact tone, style, and language guidelines defined in your soul instructions above.",
                    soul_content, global_goal
                );

                if let Ok(vic_res) = state.gemini.generate_content().with_user_message(victory_prompt).with_temperature(0.7).execute().await {
                    let raw_v = vic_res.text().trim().to_string();
                    let healed_v = crate::common::format::heal_thinking_tags(&raw_v);
                    let parsed_v = crate::common::agent::parse_llm_output(&healed_v);
                    let victory_msg = parsed_v.response.trim().to_string();

                    let saved = save_message(
                        &pool, conversation_id, "assistant", &victory_msg,
                        None, None, 0, 0, 0, None, None, None, None, None, None, None,
                        Some(&state.redis)
                    ).await?;

                    let members: Vec<Uuid> = sqlx::query_scalar(
                        "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
                    )
                    .bind(conversation_id)
                    .fetch_all(&pool)
                    .await
                    .unwrap_or_default();

                    send_message_to_subscriber(
                        &state, members, conversation_id,
                        MessageSource::Web { name: "web".to_string() },
                        saved.to_sse_json(0), saved.into()
                    ).await;
                }

                return Ok(());
            } else {
                // Advance to next step
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'step_end', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind(format!("Force-advanced from step {} to step {} after stuck turns.", current_step_index, force_next))
                .bind(json!({ "checkpoints": force_checkpoints_val }))
                .execute(&pool)
                .await;

                let _ = sqlx::query(
                    "UPDATE autonomous_tasks SET current_step_index = $1, checkpoints = $2 WHERE id = $3"
                )
                .bind(force_next)
                .bind(&force_checkpoints_val)
                .bind(task_id)
                .execute(&pool)
                .await;

                let _ = dispatch_task_update(task_id, conversation_id, &state, &pool).await;
                continue;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    // iteration_limit exhausted — mark task as failed with a clear reason
    info!("HTO: iteration_limit exhausted for task [{}]. Marking as failed.", task_id);
    let _ = sqlx::query(
        "UPDATE autonomous_tasks SET status = 'failed' WHERE id = $1"
    )
    .bind(task_id)
    .execute(&pool)
    .await;

    // Fetch conversation_id for the final dispatch (out of while-loop scope)
    if let Ok(Some(conv_id)) = sqlx::query_scalar::<_, Uuid>(
        "SELECT conversation_id FROM autonomous_tasks WHERE id = $1"
    )
    .bind(task_id)
    .fetch_optional(&pool)
    .await
    {
        let _ = dispatch_task_update(task_id, conv_id, &state, &pool).await;
    }

    Ok(())
}

fn extract_json_structure(text: &str) -> Option<Value> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            let chunk = &text[start..=end];
            if let Ok(parsed) = serde_json::from_str::<Value>(chunk) {
                if parsed.is_object() {
                    return Some(parsed);
                }
            }
        }
    }
    None
}

pub async fn dispatch_task_update(
    task_id: Uuid,
    conversation_id: Uuid,
    state: &AppState,
    pool: &sqlx::Pool<sqlx::Postgres>,
) -> anyhow::Result<()> {
    #[derive(sqlx::FromRow)]
    struct TaskDetails {
        title: String,
        status: String,
        current_step_index: i32,
        checkpoints: serde_json::Value,
    }

    let task = sqlx::query_as::<_, TaskDetails>(
        "SELECT title, status, current_step_index, checkpoints \
         FROM autonomous_tasks WHERE id = $1 LIMIT 1"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

    let cumulative_tokens = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT cumulative_tokens FROM autonomous_tasks WHERE id = $1"
    )
    .bind(task_id)
    .fetch_one(pool)
    .await
    .unwrap_or(None)
    .unwrap_or(0);

    // --- Slim checkpoints: only index + status, no description text ---
    // Keeps the MQTT packet well under 10KB even with many steps.
    let slim_checkpoints: Vec<serde_json::Value> = task.checkpoints
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|cp| serde_json::json!({
            "step_index": cp.get("step_index").or_else(|| cp.get("index")),
            "status": cp.get("status")
        }))
        .collect();

    // Lightweight signal only — NO logs, NO raw_payload, NO global_goal.
    // The frontend re-fetches /tasks/{id}/timeline via HTTP when it receives this ping.
    // This keeps every MQTT packet well under the 10KB free-tier limit.
    let payload = serde_json::json!({
        "id": task_id,
        "title": task.title,
        "status": task.status,
        "current_step_index": task.current_step_index,
        "checkpoints": slim_checkpoints,
        "cumulative_tokens": cumulative_tokens
    });

    let _ = state.dispatch(crate::services::event_dispatcher::AppEvent::conversation(
        conversation_id,
        "task_update",
        payload
    )).await;

    Ok(())
}
