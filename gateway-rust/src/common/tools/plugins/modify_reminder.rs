use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ModifyReminderParameters;
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

    fn matching_intents(&self) -> &[&str] {
        &["MODIFY_REMINDER", "UPDATE_REMINDER", "CHANGE_REMINDER_TIME", "REMINDER"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: ModifyReminderParameters = serde_json::from_value(args)?;
            info!(
                "Modifying reminder via plugin: {} with action: {}",
                params.reminder_id, params.action
            );

            let reminder_id = match Uuid::parse_str(&params.reminder_id) {
                Ok(id) => id,
                Err(e) => {
                    return Ok(format!("Invalid reminder ID: {}", e));
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
                                return Ok(format!(
                                    "Invalid snooze date format: {}. Please use ISO 8601.",
                                    e
                                ));
                            }
                        },
                        None => {
                            return Ok("Snooze action requires 'snooze_until' parameter.".to_string());
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
                    return Ok(format!("Invalid action: {}", params.action));
                }
            };

            match result {
                Ok(_) => Ok(format!("Reminder {} successfully.", params.action)),
                Err(e) => Ok(format!("Failed to modify reminder: {}", e)),
            }
        }
        .boxed()
    }
}
