use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, GetReminderStatsParameters};
use crate::common::tools::ToolDispatcher;
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::{json, Value};
use tracing::info;

pub struct GetReminderStatsPlugin;

impl NomiToolPlugin for GetReminderStatsPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "get_reminder_stats",
                "Get stats about existing reminders, optionally filtered by DateTime ranges. Examples: 'What's left for the rest of the day?', 'Any reminders for this weekend?'",
                None,
            )
            .with_parameters::<GetReminderStatsParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        "### REMINDER LOGIC\n- Use `schedule_task`, `modify_reminder`, `get_reminder_stats` to manage schedule. Use relative analysis to translate vague human terms into precise Datetimes.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["GET_REMINDER_STATS", "VIEW_REMINDER_ANALYTICS", "CHECK_REMINDER_HISTORY", "REMINDER", "DASHBOARD"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: GetReminderStatsParameters = serde_json::from_value(args)?;
            info!("Executing get_reminder_stats via plugin");

            let user_id = match dispatcher.user_id {
                Some(id) => id,
                None => {
                    return Ok(ToolResult {
                        error: "User ID not found in context".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let start_after = match params.start_after {
                Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(dt) => Some(dt.with_timezone(&Utc)),
                    Err(e) => {
                        return Ok(ToolResult {
                            error: format!("Invalid start_after format: {}. Please use ISO 8601.", e),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: "".to_string(),
                            ref_id: "".to_string(),
                        });
                    }
                },
                None => None,
            };

            let end_before = match params.end_before {
                Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                    Ok(dt) => Some(dt.with_timezone(&Utc)),
                    Err(e) => {
                        return Ok(ToolResult {
                            error: format!("Invalid end_before format: {}. Please use ISO 8601.", e),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: "".to_string(),
                            ref_id: "".to_string(),
                        });
                    }
                },
                None => None,
            };

            let limit = params.limit.unwrap_or(20) as i64;

            let query_result = sqlx::query!(
                r#"
                SELECT 
                    id,
                    COALESCE(payload->>'message', content) as "content!",
                    (due_at AT TIME ZONE 'Asia/Jakarta') as due_at,
                    status,
                    frequency,
                    current_runs
                FROM reminders
                WHERE user_id = $1
                  AND task_type = 'REMINDER'
                  AND ($2::TIMESTAMPTZ IS NULL OR due_at >= $2)
                  AND ($3::TIMESTAMPTZ IS NULL OR due_at <= $3)
                  AND ($4::TEXT IS NULL OR status = $4)
                ORDER BY due_at ASC
                LIMIT $5;
                "#,
                user_id,
                start_after,
                end_before,
                params.status_filter,
                limit
            )
            .fetch_all(&dispatcher.pool)
            .await;

            match query_result {
                Ok(rows) => {
                    let mut results = Vec::new();
                    let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
                    for row in rows {
                        let due_at_naive = row.due_at.unwrap();
                        let due_at_utc = tz
                            .from_local_datetime(&due_at_naive)
                            .single()
                            .unwrap()
                            .with_timezone(&Utc);
                        let item = json!({
                            "id": row.id.to_string(),
                            "content": row.content,
                            "due_at_utc": due_at_utc.to_rfc3339(),
                            "due_at_local": due_at_naive.format("%Y-%m-%d %H:%M:%S").to_string(),
                            "status": row.status,
                            "frequency": row.frequency,
                            "current_runs": row.current_runs
                        });
                        results.push(item);
                    }

                    let content = if results.is_empty() {
                        "No reminders found for the given criteria.".to_string()
                    } else {
                        serde_json::to_string_pretty(&results).unwrap_or_default()
                    };

                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content,
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
                Err(e) => Ok(ToolResult {
                    error: format!("Database error fetching reminders: {}", e),
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
