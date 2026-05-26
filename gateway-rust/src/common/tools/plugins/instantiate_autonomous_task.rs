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
            "description": "Spawns an autonomous, background-threaded agent loop to execute a multi-step task or order. Returns the task UUID, which is saved in metadata.",
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
         1. Call this tool when the user requests a complex, multi-step chore (e.g. booking, ordering, searching, scheduling, contacting team members).\n\
         2. Always formulate a sequential plan divided into logical checkpoints before invoking this tool.\n\
         3. Ensure the source_message_id maps precisely to the triggering message UUID to enable rich history context."
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
            let parsed_source_msg = Uuid::parse_str(source_msg_str)?;

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
                follow_up_prompt: format!("Inform Trian in a warm, highly casual Indonesian/English slang update that the background autonomous task for '{}' has been launched, and they can watch live progress in the timeline side-panel. Keep it fun and teammate-oriented (e.g. 'aman', 'otw', 'sip', 'gua').", title),
                ref_id: task_uuid.to_string(), // Triggers orchestrator metadata hook
            })
        }
        .boxed()
    }
}
