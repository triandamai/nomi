use crate::common::tools::ToolDispatcher;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use crate::services::task_orchestrator::spawn_task_loop;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct InstantiateAutonomousTaskPlugin;

impl NomiToolPlugin for InstantiateAutonomousTaskPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "instantiate_autonomous_task",
            "description": "Spawns and STARTS an autonomous background worker/agent loop IMMEDIATELY (now) to execute a multi-step task, project, background chore, monitoring task, booking, research pipeline, or messaging order. Use this tool ONLY when the user wants to start or try again a task now in the background. DO NOT call 'schedule_task' unless you need to delay the execution to a specific date/time in the future. You MUST pass: 'task_title' (NOT 'task_name'), 'global_goal' (NOT 'description'), and 'checkpoints' (NOT 'steps') exactly as specified.",
            "parameters": {
                "type": "object",
                "properties": {
                    "task_title": {
                        "type": "string",
                        "description": "A short, descriptive title for the task (e.g., 'Book Table at Sederhana')."
                    },
                    "global_goal": {
                        "type": "string",
                        "description": "The complete, detailed description of the final objective Nomi must achieve."
                    },
                    "source_message_id": {
                        "type": "string",
                        "description": "The exact database UUID string of the message where this instruction originated."
                    },
                    "checkpoints": {
                        "type": "array",
                        "description": "An ordered array of step objects representing the sequential checklist plan.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "step_index": { "type": "integer" },
                                "action_objective": { "type": "string" },
                                "status": { "type": "string", "enum": ["pending", "completed", "failed"] }
                            },
                            "required": ["step_index", "action_objective", "status"]
                        }
                    }
                },
                "required": ["task_title", "global_goal", "source_message_id", "checkpoints"]
            }
        })
    }

    fn rules(&self) -> &str {
        "\n## AUTONOMOUS TASK LAUNCH RULES:\n\
         1. Proactively call this tool when the user requests a multi-step project, chore, background research, monitoring task, booking, or messaging sequence. \
            Do NOT wait for the user to explicitly ask you to start an autonomous task or wait for confirmation. If the request is complex, multi-step, or needs background/proactive operations, decide to launch it immediately!\n\
         2. STRICT PARAMETER MAPPING: You MUST use the exact schema parameter names: 'task_title' (NOT 'task_name'), 'global_goal' (NOT 'description' or 'goal'), 'source_message_id', and 'checkpoints' (NOT 'steps'). Using incorrect parameter names will fail instantly!\n\
         3. IMMEDIATE VS SCHEDULED: Use this tool to start execution IMMEDIATELY. If the user wants to schedule a task in the future (e.g. 'at 9 PM' or 'tomorrow'), use 'schedule_task' instead.\n\
         4. Always formulate a sequential plan divided into logical checkpoints before invoking this tool.\n\
         5. Ensure the source_message_id maps precisely to the triggering message UUID to enable rich history context."
    }

    fn matching_intents(&self) -> &[&str] {
        &["CORE_SYSTEM_TASK_INIT", "AUTONOMOUS_TASK", "BOOKING", "ORDERING"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let title = args["task_title"].as_str().ok_or_else(|| anyhow::anyhow!("Missing task_title parameter"))?;
            let goal = args["global_goal"].as_str().ok_or_else(|| anyhow::anyhow!("Missing global_goal parameter"))?;
            let source_msg_str = args["source_message_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing source_message_id parameter"))?;
            let checkpoints = args["checkpoints"].clone();

            let conversation_id = dispatcher.conversation_id.ok_or_else(|| anyhow::anyhow!("No active conversation context"))?;

            // Robust validation of source_message_id: Check existence and fallback to prevent foreign key violations
            let mut parsed_source_msg: Option<Uuid> = None;
            if let Ok(parsed_uuid) = Uuid::parse_str(source_msg_str) {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1)"
                )
                .bind(parsed_uuid)
                .fetch_one(&dispatcher.pool)
                .await
                .unwrap_or(false);

                if exists {
                    parsed_source_msg = Some(parsed_uuid);
                }
            }

            // Fallback to the most recent message in this conversation if not found
            if parsed_source_msg.is_none() {
                let latest_msg_id: Option<Uuid> = sqlx::query_scalar(
                    "SELECT id FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 1"
                )
                .bind(conversation_id)
                .fetch_optional(&dispatcher.pool)
                .await
                .unwrap_or(None);

                parsed_source_msg = latest_msg_id;
            }

            // --- DEDUPLICATION & RECURSION PREVENTION LOCKS ---
            // 1. Prevent duplicate tasks for the exact same source triggering message
            if let Some(source_msg_id) = parsed_source_msg {
                let existing_task = sqlx::query(
                    "SELECT id, status FROM autonomous_tasks WHERE source_message_id = $1 LIMIT 1"
                )
                .bind(source_msg_id)
                .fetch_optional(&dispatcher.pool)
                .await?;

                if let Some(row) = existing_task {
                    use sqlx::Row;
                    let task_uuid: Uuid = row.get("id");
                    let status: String = row.get("status");

                    let res_content = format!(
                        "Blocked: An autonomous task has already been spawned for source message ID '{}'. \
                         Existing Task ID: '{}' has status '{}'. Duplicate prevention is active to prevent recursion loops.",
                        source_msg_id, task_uuid, status
                    );
                    return Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: res_content,
                        follow_up_prompt: format!(
                            "Explain to the user that the background workflow for '{}' was already initiated (Task ID: '{}', status: '{}'). \
                             If it failed with an error, do NOT try to spawn it again. Ask the user if they want you to retry/restart the existing task or how they want to proceed.",
                            title, task_uuid, status
                        ),
                        ref_id: task_uuid.to_string(),
                    });
                }
            }

            // 2. Prevent duplicate concurrent active tasks with the same title in the same conversation
            let active_task = sqlx::query(
                "SELECT id, status FROM autonomous_tasks \
                 WHERE conversation_id = $1 AND title = $2 \
                 AND status IN ('running', 'paused_for_input', 'waiting_external_feedback') LIMIT 1"
            )
            .bind(conversation_id)
            .bind(title)
            .fetch_optional(&dispatcher.pool)
            .await?;

            if let Some(row) = active_task {
                use sqlx::Row;
                let task_uuid: Uuid = row.get("id");
                let status: String = row.get("status");

                let res_content = format!(
                    "Blocked: A background task with the title '{}' is already active in this conversation. \
                     Active Task ID: '{}', current status: '{}'.",
                    title, task_uuid, status
                );
                return Ok(ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: res_content,
                    follow_up_prompt: format!(
                        "Inform the user that the background task '{}' (Task ID: '{}') is already active in this conversation with status '{}'. \
                         Explain that they can watch its progress live in the side-panel and that you cannot spawn a duplicate active task.",
                        title, task_uuid, status
                    ),
                    ref_id: task_uuid.to_string(),
                });
            }
            // --------------------------------------------------

            // 1. Write the new task to the main ledger database using non-macro query_scalar
            let task_uuid = sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO autonomous_tasks (conversation_id, source_message_id, title, global_goal, status, current_step_index, checkpoints) \
                 VALUES ($1, $2, $3, $4, 'running', 0, $5) RETURNING id"
            )
            .bind(conversation_id)
            .bind(parsed_source_msg)
            .bind(title)
            .bind(goal)
            .bind(checkpoints.clone())
            .fetch_one(&dispatcher.pool)
            .await?;

            // 2. Log 'step_start' event into autonomous_task_logs using non-macro query
            let _ = sqlx::query(
                "INSERT INTO autonomous_task_logs (task_id, step_index, event_type, log_content, raw_payload) \
                 VALUES ($1, 0, 'step_start', $2, $3)"
            )
            .bind(task_uuid)
            .bind(format!("Autonomous task launched: {}", title))
            .bind(json!({ "checkpoints": checkpoints }))
            .execute(&dispatcher.pool)
            .await;


            // 3. Spawn the background loop in a separate Tokio task thread
            spawn_task_loop(dispatcher.app_state.clone(), task_uuid);

            let res_content = format!("Autonomous task loop successfully ignited for '{}' with Task ID: {}", title, task_uuid);
            Ok(ToolResult {
                error: "".to_string(),
                success: true,
                content: res_content,
                follow_up_prompt: format!("Inform the user that the background Nomi Workflow for '{}' has been launched, and they can watch live progress in the Nomi Workflow timeline side-panel.", title),
                ref_id: task_uuid.to_string(), // Triggers orchestrator metadata hook
            })
        }
        .boxed()
    }
}
