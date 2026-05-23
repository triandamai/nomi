use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, ModifyReminderParameters};
use crate::common::tools::ToolDispatcher;
use chrono::Utc;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;
use uuid::Uuid;

pub struct ModifyReminderPlugin;

impl NomiToolPlugin for ModifyReminderPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "modify_reminder",
                "Modify an existing reminder: snooze it to a new time, cancel it, or mark it as done.",
                None,
            )
            .with_parameters::<ModifyReminderParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        "### REMINDER LOGIC\n- Use `schedule_task`, `modify_reminder`, `get_reminder_stats` to manage schedule. Use relative analysis to translate vague human terms into precise Datetimes.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["MODIFY_REMINDER", "UPDATE_REMINDER", "CHANGE_REMINDER_TIME", "REMINDER"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: ModifyReminderParameters = serde_json::from_value(args)?;
            info!(
                "Modifying reminder via plugin: {} with action: {}",
                params.reminder_id, params.action
            );

            let reminder_id = match Uuid::parse_str(&params.reminder_id) {
                Ok(id) => id,
                Err(e) => {
                    return Ok(ToolResult {
                        error: format!("Invalid reminder ID: {}", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let result = match params.action.as_str() {
                "done" | "completed" => {
                    sqlx::query!(
                        "UPDATE reminders SET status = 'completed', updated_at = NOW() WHERE id = $1",
                        reminder_id
                    )
                    .execute(&dispatcher.pool)
                    .await
                }
                "cancel" | "archived" => {
                    sqlx::query!(
                        "UPDATE reminders SET status = 'archived', updated_at = NOW() WHERE id = $1",
                        reminder_id
                    )
                    .execute(&dispatcher.pool)
                    .await
                }
                "snooze" => {
                    let snooze_until = match params.snooze_until {
                        Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                            Ok(dt) => dt.with_timezone(&Utc),
                            Err(e) => {
                                return Ok(ToolResult {
                                    error: format!("Invalid snooze date format: {}. Please use ISO 8601.", e),
                                    success: false,
                                    content: "".to_string(),
                                    follow_up_prompt: "".to_string(),
                                    ref_id: "".to_string(),
                                });
                            }
                        },
                        None => {
                            return Ok(ToolResult {
                                error: "Snooze action requires 'snooze_until' parameter.".to_string(),
                                success: false,
                                content: "".to_string(),
                                follow_up_prompt: "".to_string(),
                                ref_id: "".to_string(),
                            });
                        }
                    };

                    sqlx::query!(
                        "UPDATE reminders SET due_at = $1, status = 'pending', snooze_count = snooze_count + 1, updated_at = NOW() WHERE id = $2",
                        snooze_until,
                        reminder_id
                    )
                    .execute(&dispatcher.pool)
                    .await
                }
                _ => {
                    return Ok(ToolResult {
                        error: format!("Invalid action: {}", params.action),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            match result {
                Ok(_) => Ok(ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: format!("Reminder {} successfully.", params.action),
                    follow_up_prompt: "".to_string(),
                    ref_id: reminder_id.to_string(),
                }),
                Err(e) => Ok(ToolResult {
                    error: format!("Failed to modify reminder: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
            }
        }
        .boxed()
    }
}
