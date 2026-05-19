use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ScheduleTaskParameters;
use crate::common::tools::ToolDispatcher;
use chrono::{NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;

pub struct ScheduleTaskPlugin;

impl NomiToolPlugin for ScheduleTaskPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "schedule_task",
                "Schedule a background task (Reminders, DMs, or Agent actions). Recurrence (daily, weekly, monthly) is supported. The `due_at` field has a STRICT FORMAT: 'YYYY-MM-DD HH:MM'. Explicitly calculate this timestamp relative to the [SYSTEM TIME ANCHOR] provided in the system prompt. For example, if current time is 2026-05-14 20:27 WIB and the user says 'in 10 minutes', you MUST output exactly '2026-05-14 20:37'. Never include trailing 'Z', offsets, or seconds. The user is in WIB (UTC+7).",
                None,
            )
            .with_parameters::<ScheduleTaskParameters>()
        ).unwrap()
    }

    fn matching_intents(&self) -> &[&str] {
        &["SCHEDULE_TASK", "CREATE_REMINDER", "SET_ALARM", "REMINDER"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: ScheduleTaskParameters = serde_json::from_value(args)?;
            info!("Scheduling task via plugin: {:?}", params.task_type);

            let user_id = match dispatcher.user_id {
                Some(id) => id,
                None => {
                    return Ok("User ID not found in context".to_string());
                }
            };

            let tz_wib: Tz = "Asia/Jakarta".parse().unwrap();
            let due_at_utc = match NaiveDateTime::parse_from_str(&params.due_at, "%Y-%m-%d %H:%M") {
                Ok(naive) => {
                    // Assume LLM sent time in WIB (UTC+7)
                    match tz_wib.from_local_datetime(&naive).single() {
                        Some(dt) => dt.with_timezone(&Utc),
                        None => {
                            return Ok("Ambiguous or invalid time for WIB timezone".to_string());
                        }
                    }
                }
                Err(e) => {
                    return Ok(format!(
                        "Invalid date format: {}. Please use 'YYYY-MM-DD HH:MM'.",
                        e
                    ));
                }
            };

            let due_at_wib = due_at_utc.with_timezone(&tz_wib);
            let frequency = params.frequency.unwrap_or_else(|| "once".to_string());

            let task_description = match params.task_type.to_uppercase().as_str() {
                "REMINDER" => format!(
                    "reminder: '{}'",
                    params
                        .payload
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No description")
                ),
                "SEND_DM" => {
                    let recipient = params
                        .payload
                        .get("recipient_jid")
                        .and_then(|v| v.as_str())
                        .unwrap_or("someone");
                    format!("automated DM to be sent to {}", recipient)
                }
                "TRIGGER_AGENT" => "background agent execution".to_string(),
                _ => format!("task of type {}", params.task_type),
            };

            let result = sqlx::query!(
                "INSERT INTO reminders (user_id, conversation_id, task_type, payload, due_at, frequency, max_repeats, content)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
                user_id,
                dispatcher.conversation_id,
                params.task_type.to_uppercase(),
                params.payload,
                due_at_utc,
                frequency,
                params.max_repeats,
                task_description.clone(),
            )
            .fetch_one(&dispatcher.pool)
            .await;

            match result {
                Ok(_) => {
                    // Get user name for personalized response
                    let display_name: String = sqlx::query_scalar(
                        "SELECT COALESCE(display_name, 'Trian') FROM users WHERE id = $1",
                    )
                    .bind(user_id)
                    .fetch_one(&dispatcher.pool)
                    .await
                    .unwrap_or_else(|_| "Trian".to_string());

                    let content = format!(
                        "Got it, {}! I've scheduled your {} for {}.",
                        display_name,
                        task_description,
                        due_at_wib.format("%H:%M WIB").to_string()
                    );

                    Ok(content)
                }
                Err(e) => Ok(format!("Failed to schedule task: {}", e)),
            }
        }
        .boxed()
    }
}
