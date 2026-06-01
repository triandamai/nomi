use crate::common::repository::message_repo::save_message;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;
use crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber;
use crate::feature::MessageSource;
use crate::services::event_dispatcher::AppEvent;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use tracing::info;

/// A dedicated HTO tool that delivers the final task result directly back to the
/// owner's conversation — the exact chat where the autonomous task was originally
/// created. No user search, no UUID guessing, no channel resolution needed.
pub struct ReportToOwnerPlugin;

impl NomiToolPlugin for ReportToOwnerPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "report_to_owner",
            "description": "Delivers a message or result summary directly to the owner (Trian) in the conversation where this autonomous task was originally created. Use this as the FINAL step of every autonomous task to inform the owner of the outcome. You do NOT need to search for the user or resolve any UUID — the system automatically knows who the owner is and which conversation to deliver to.",
            "parameters": {
                "type": "object",
                "properties": {
                    "message_body": {
                        "type": "string",
                        "description": "The full result summary or notification message to deliver to the task owner. Write it conversationally as Nomi would speak to Trian."
                    }
                },
                "required": ["message_body"]
            }
        })
    }

    fn rules(&self) -> &str {
        "\n## REPORT TO OWNER — AUTONOMOUS TASK RULE:\n\
         - At the FINAL step of every autonomous task, you MUST call `report_to_owner` to deliver\n\
           the result back to the task creator (Trian) in the original conversation.\n\
         - Do NOT use `send_message` or `manage_user` to notify the owner — use `report_to_owner` instead.\n\
         - This tool automatically resolves the correct conversation and owner UUID from the task context.\n\
         - After calling `report_to_owner`, output your checkpoint completion JSON to close the task."
    }

    fn matching_intents(&self) -> &[&str] {
        &["HTO_WORKFLOW_REGISTRY", "COMMUNICATION"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let message_body = match args["message_body"].as_str() {
                Some(b) => b,
                None => {
                    return Ok(ToolResult {
                        success: false,
                        error: "Missing required parameter: message_body".to_string(),
                        content: "".to_string(),
                        follow_up_prompt: "Please provide a message_body to deliver to the owner.".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            // Resolve owner conversation — this is the task's parent conversation,
            // already injected into the dispatcher when the HTO loop started.
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok(ToolResult {
                        success: false,
                        error: "No active conversation context found in task dispatcher.".to_string(),
                        content: "".to_string(),
                        follow_up_prompt: "The task has no parent conversation linked. This is a system configuration issue.".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let owner_user_id = dispatcher.user_id;

            info!(
                "report_to_owner: delivering result to conversation={} owner={:?}",
                conversation_id, owner_user_id
            );

            // 1. Persist message to DB via the proper save_message path
            let saved = save_message(
                &dispatcher.pool,
                conversation_id,
                "assistant",
                message_body,
                None, // thought
                None, // user_id (sender is Nomi/system)
                0, 0, 0,
                None, None, None, None, None, None, None,
                Some(&dispatcher.app_state.redis),
            )
            .await;

            let message_id = match &saved {
                Ok(msg) => msg.id,
                Err(e) => {
                    return Ok(ToolResult {
                        success: false,
                        error: format!("Failed to persist message to DB: {}", e),
                        content: "".to_string(),
                        follow_up_prompt: "Database write failed. Retry or check DB connectivity.".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let saved_msg = saved.unwrap();

            // 2. Collect all conversation members so they all receive the MQTT push
            let members: Vec<uuid::Uuid> = sqlx::query_scalar(
                "SELECT user_id FROM conversation_members WHERE conversation_id = $1"
            )
            .bind(conversation_id)
            .fetch_all(&dispatcher.pool)
            .await
            .unwrap_or_default();

            // 3. Push via user-scoped MQTT topics (nomi/users/{id}/message)
            send_message_to_subscriber(
                &dispatcher.app_state,
                members,
                conversation_id,
                MessageSource::Web { name: "web".to_string() },
                saved_msg.to_sse_json(0),
                saved_msg.into(),
            )
            .await;

            // 4. Also push on the conversation-scoped topic (nomi/conversations/{id}/message)
            // as a belt-and-suspenders guarantee that the frontend receives it.
            let _ = dispatcher.app_state.dispatch(
                AppEvent::conversation(
                    conversation_id,
                    "message",
                    json!({
                        "id": message_id,
                        "role": "assistant",
                        "content": message_body,
                        "conversation_id": conversation_id,
                    }),
                )
            ).await;

            info!("report_to_owner: message delivered successfully [msg_id={}]", message_id);

            Ok(ToolResult {
                success: true,
                error: "".to_string(),
                content: format!(
                    "Message successfully delivered to owner in conversation {}.",
                    conversation_id
                ),
                follow_up_prompt: format!(
                    "You have successfully reported the task result to Trian in the original conversation. \
                     Now output your checkpoint completion JSON to finalize the task."
                ),
                ref_id: format!("MSG:{}", message_id),
            })
        }
        .boxed()
    }
}
