use crate::common::app_state::AppState;
use crate::common::tools::ToolDispatcher;
use crate::common::repository::message_repo::save_message;
use crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber;
use crate::feature::MessageSource;
use crate::rag::get_embedding;
use serde_json::{json, Value};
use tracing::{error, info};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct TaskInfo {
    conversation_id: Uuid,
    title: String,
    global_goal: String,
    checkpoints: Value,
    status: String,
    source_message_id: Option<Uuid>,
    current_step_index: i32,
    sub_conversation_id: Option<Uuid>,
    soul_content: Option<String>,
    bootstrap_content: Option<String>,
}

#[derive(sqlx::FromRow)]
struct HistoricalTurn {
    role: String,
    content: String,
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
    let mut iteration_limit = 15;

    while iteration_limit > 0 {
        iteration_limit -= 1;

        // 1. Fetch live task coordinates along with room configuration and active persona using non-macro query_as
        let task_row = sqlx::query_as::<_, TaskInfo>(
            "SELECT t.conversation_id, t.title, t.global_goal, t.checkpoints, t.status, t.source_message_id, t.current_step_index, \
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
        let source_message_id = task.source_message_id.unwrap_or(Uuid::nil());
        let mut sub_conversation_id = task.sub_conversation_id;

        // 2. Fetch chronological message context leading up to task initiation using non-macro query_as
        let historical_turns = if source_message_id != Uuid::nil() {
            let turns = sqlx::query_as::<_, HistoricalTurn>(
                "SELECT role, content FROM ( \
                     SELECT role, content, created_at FROM messages \
                     WHERE conversation_id = $1 AND created_at <= (SELECT created_at FROM messages WHERE id = $2) \
                     ORDER BY created_at DESC LIMIT 10 \
                 ) sub ORDER BY created_at ASC"
            )
            .bind(conversation_id)
            .bind(source_message_id)
            .fetch_all(&pool)
            .await?;

            turns
                .into_iter()
                .map(|m| format!("[{}]: {}", m.role, m.content))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            "".to_string()
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

        // Perform RAG vector lookup for user context preferences
        let mut rag_context = String::new();
        if let Ok(embedding_res) = get_embedding(&state.gemini_api_key, &global_goal).await {
            if let Ok(similar_records) = crate::rag::search_similar(&pool, embedding_res.embedding.values, 3).await {
                for record in similar_records {
                    rag_context.push_str(&format!("* {}\n", record.content));
                }
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
             HISTORICAL CONTEXT LEADING UP TO THIS TASK:\n\
             {}\n\n\
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
             4. If you need input from the user (such as a missing parameter like timing, or explicit approval for payments), transition the task status by outputting a status update with status=\"paused_for_input\" and detailing the required form inputs using `<RequiredField>` tags.\n\
             5. When the current step index is completed, output a JSON structure explaining the progress updates to update checkpoints.\n\
             6. If ALL steps in the HTO plan are fully completed and the global goal is satisfied, mark the final task status as \"completed\".\n\
             7. MANDATORY RULE: Never guess, hallucinate, or pass phone numbers, handles, JIDs or numeric strings (like '@2297908166856') to the 'user_id' parameter of `send_message`. You MUST always invoke the search tool (`manage_user` with action='search') first to find the user's database UUID, then pass that UUID directly into the 'user_id' parameter of `send_message`. Any target parameter that is not a valid 36-character UUID will fail instantly.\n\n\
             Respond in a highly structured manner.",
            soul_text,
            bootstrap_text,
            user_display_name,
            user_email,
            rag_context,
            historical_turns,
            global_goal,
            current_step_index,
            checkpoints,
            scratchpad,
            current_step_index
        );

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

        info!("Sending operational prompt to background LLM loop via common::agent::send_prompt...");
        
        let (response, _) = crate::common::agent::send_prompt(
            &dispatcher,
            crate::common::agent::agent_model::PromptActor::User {
                history: "".to_string(),
                memories: "".to_string(),
                message: "Evaluate scratchpad, execute the next tool, or update checkpoints plan.".to_string(),
                system_prompt: hto_prompt,
                media: None,
            },
            &["FULL_REGISTRY".to_string()],
        )
        .await
        .map_err(|e| anyhow::anyhow!("LLM error: {}", e))?;

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

        let response_text = response.text().trim().to_string();
        info!("Background LLM response: {}", response_text);

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
                
                // Write 'tool_execution' timeline event
                let _ = sqlx::query(
                    "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                     VALUES ($1, $2, 'tool_execution', $3, $4)"
                )
                .bind(task_id)
                .bind(current_step_index)
                .bind(format!("Tool [{}] executed. Result success: {}. Response: {}", name, result.success, result.content))
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
                        
                        // Spawn isolated conversation row using non-macro query_scalar
                        let sub_title = format!("{} (Sub-chat for Task)", task.title);
                        let sub_convo_id = sqlx::query_scalar::<_, Uuid>(
                            "INSERT INTO conversations (title, conversation_type, soul_content, bootstrap_content) \
                             VALUES ($1, 'channel_subchat', $2, $3) RETURNING id"
                        )
                        .bind(sub_title)
                        .bind(soul_text.clone())
                        .bind(bootstrap_text.clone())
                        .fetch_one(&pool)
                        .await?;

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
                     Explain that you stopped it to save resources, and suggest they check lookup details or parameters. \
                     Adopt the exact tone, style, and language guidelines defined in your soul instructions above.",
                    soul_content, task.title, current_step_index
                );

                if let Ok(err_res) = state.gemini.generate_content().with_user_message(error_prompt).with_temperature(0.7).execute().await {
                    let err_msg = err_res.text().trim().to_string();
                    
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

            // Continue the loop to evaluate the tool outcomes immediately
            continue;
        }

        // B. Handle Interactive Input Pauses / Clarifications
        if response_text.contains("status=\"paused_for_input\"") || response_text.contains("<RequiredField") {
            info!("HTO loop: background planner requests input clarification/approval. Pausing.");
            
            // Extract natural message text to show Trian Damai
            let cleaned_xml = response_text.clone();
            
            // Refine conversational text to Nomi slang style
            let slang_prompt = format!(
                "You are Nomi chatting casually with your friend Trian in a messaging thread. Convert this raw planning text into a natural human update explaining what you need confirmation/input for: \"{}\". \
                 Keep it casual—Indonesian/English teammate slang style is highly encouraged (e.g. 'aman', 'otw', 'sip', 'gua').",
                cleaned_xml
            );
            
            let slang_res = state.gemini.generate_content()
                .with_user_message(slang_prompt)
                .with_temperature(0.7)
                .execute()
                .await?;
                
            let natural_text = slang_res.text().trim().to_string();

            // Save telemetry message with XML details to primary room
            let saved = save_message(
                &pool,
                conversation_id,
                "assistant",
                &format!("{}\n\n{}", natural_text, cleaned_xml),
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
            let _ = sqlx::query(
                "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                 VALUES ($1, $2, 'outbound_msg', $3, $4)"
            )
            .bind(task_id)
            .bind(current_step_index)
            .bind(format!("Task paused for input: {}", natural_text))
            .bind(json!({ "raw_xml": cleaned_xml }))
            .execute(&pool)
            .await;

            // Set task status in DB to 'paused_for_input'
            let _ = sqlx::query(
                "UPDATE autonomous_tasks SET status = 'paused_for_input' WHERE id = $1"
            )
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

        // Substring-based fallback matches for completion indicators
        let response_lower = response_text.to_lowercase();
        if response_lower.contains("status=\"completed\"") 
            || response_lower.contains("\"status\": \"completed\"")
            || response_lower.contains("\"status\":\"completed\"")
            || response_lower.contains("completed final goal")
            || response_lower.contains("completed the task")
        {
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
            if next_step as usize >= total_steps 
                || response_lower.contains("completed final goal")
                || response_lower.contains("task completed")
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
                     Inform the user that you have fully completed the global goal: '{}'. \
                     Celebrate this milestone! \
                     Adopt the exact tone, style, and language guidelines defined in your soul instructions above.",
                    soul_content, global_goal
                );
                
                if let Ok(vic_res) = state.gemini.generate_content().with_user_message(victory_prompt).with_temperature(0.7).execute().await {
                    let victory_msg = vic_res.text().trim().to_string();
                    
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

        // Catch-all: default iteration yield to avoid tight loops
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
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
        global_goal: String,
        status: String,
        current_step_index: i32,
        checkpoints: serde_json::Value,
    }

    let task = sqlx::query_as::<_, TaskDetails>(
        "SELECT title, global_goal, status, current_step_index, checkpoints \
         FROM autonomous_tasks WHERE id = $1 LIMIT 1"
    )
    .bind(task_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

    // Dynamically retrieve cumulative token counter with fallback if migration not yet applied
    let cumulative_tokens = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT cumulative_tokens FROM autonomous_tasks WHERE id = $1"
    )
    .bind(task_id)
    .fetch_one(pool)
    .await
    .unwrap_or(None)
    .unwrap_or(0);

    #[derive(sqlx::FromRow)]
    struct TimelineRouteLog {
        step_index: i32,
        event_type: String,
        log_content: String,
        raw_payload: serde_json::Value,
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let logs = sqlx::query_as::<_, TimelineRouteLog>(
        "SELECT step_index, event_type, log_content, raw_payload, created_at \
         FROM autonomous_task_logs WHERE task_id = $1 ORDER BY created_at ASC"
    )
    .bind(task_id)
    .fetch_all(pool)
    .await?;

    let json_logs: Vec<serde_json::Value> = logs
        .into_iter()
        .map(|log| {
            serde_json::json!({
                "step_index": log.step_index,
                "event_type": log.event_type,
                "log_content": log.log_content,
                "raw_payload": log.raw_payload,
                "created_at": log.created_at
            })
        })
        .collect();

    let payload = serde_json::json!({
        "id": task_id,
        "title": task.title,
        "global_goal": task.global_goal,
        "status": task.status,
        "current_step_index": task.current_step_index,
        "checkpoints": task.checkpoints,
        "cumulative_tokens": cumulative_tokens,
        "logs": json_logs
    });

    let _ = state.dispatch(crate::services::event_dispatcher::AppEvent::conversation(
        conversation_id,
        "task_update",
        payload
    )).await;

    Ok(())
}
