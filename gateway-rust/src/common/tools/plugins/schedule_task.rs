use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ScheduleTaskParameters, ToolResult};
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
                "Schedule a future or recurring background task (Reminders, DMs, or Autonomous Tasks to run in the FUTURE). DO NOT use this tool to start or run an autonomous task immediately (now). If the user wants a task started immediately (or says 'start a task', 'try again', or requests a task today), you MUST call 'instantiate_autonomous_task' instead! The `due_at` field is strictly required and must have a STRICT FORMAT: 'YYYY-MM-DD HH:MM'. Explicitly calculate this timestamp relative to the [SYSTEM TIME ANCHOR] provided in the system prompt. For example, if current time is 2026-05-14 20:27 WIB and the user says 'in 10 minutes', you MUST output exactly '2026-05-14 20:37'. Never include trailing 'Z', offsets, or seconds. The user is in WIB (UTC+7). If the user's scheduling instruction or target time is still ambiguous, do NOT execute this tool; instead, ask the user for clarification conversational.",
                None,
            )
            .with_parameters::<ScheduleTaskParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        "### REMINDER & SCHEDULING LOGIC\n- Use `schedule_task`, `modify_reminder`, `get_reminder_stats` to manage schedule. Use relative analysis to translate vague human terms into precise Datetimes.\n- CRITICAL: If the scheduling instruction, target time, or task goal is ambiguous, you MUST NOT proceed with scheduling. Ask the user for clarification instead.\n- STRICT DIFFERENTIATION: NEVER use `schedule_task` to start an immediate autonomous task. If the user wants an autonomous task started immediately (now), always use `instantiate_autonomous_task`.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["SCHEDULE_TASK", "CREATE_REMINDER", "SET_ALARM", "REMINDER"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: ScheduleTaskParameters = serde_json::from_value(args)?;
            info!("Scheduling task via plugin: {:?}", params.task_type);

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

            let tz_wib: Tz = "Asia/Jakarta".parse().unwrap();
            let due_at_utc = match NaiveDateTime::parse_from_str(&params.due_at, "%Y-%m-%d %H:%M") {
                Ok(naive) => {
                    // Assume LLM sent time in WIB (UTC+7)
                    match tz_wib.from_local_datetime(&naive).single() {
                        Some(dt) => dt.with_timezone(&Utc),
                        None => {
                            return Ok(ToolResult {
                                error: "Ambiguous or invalid time for WIB timezone".to_string(),
                                success: false,
                                content: "".to_string(),
                                follow_up_prompt: "".to_string(),
                                ref_id: "".to_string(),
                            });
                        }
                    }
                }
                Err(e) => {
                    return Ok(ToolResult {
                        error: format!("Invalid date format: {}. Please use 'YYYY-MM-DD HH:MM'.", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let due_at_wib = due_at_utc.with_timezone(&tz_wib);
            let frequency = params.frequency.unwrap_or_else(|| "once".to_string());

            // --- RECURSION PREVENTION LOCK FOR SCHEDULING ---
            let task_type_upper = params.task_type.to_uppercase();
            if task_type_upper == "AUTONOMOUS_TASK" || task_type_upper == "TRIGGER_AGENT" {
                if let Some(conversation_id) = dispatcher.conversation_id {
                    let active_exists = sqlx::query_scalar::<_, bool>(
                        "SELECT EXISTS(SELECT 1 FROM autonomous_tasks WHERE conversation_id = $1 AND status = 'running')"
                    )
                    .bind(conversation_id)
                    .fetch_one(&dispatcher.pool)
                    .await
                    .unwrap_or(false);

                    if active_exists {
                        return Ok(ToolResult {
                            error: format!(
                                "Blocked: Cannot schedule a new background workflow of type '{}' from inside an already running background task.",
                                task_type_upper
                            ),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: format!(
                                "BLOCKED: You attempted to schedule a recursive background task of type '{}', which is forbidden inside an active HTO workflow. \
                                 DO NOT retry schedule_task. DO NOT call any other tool. \
                                 If you have already gathered all the information needed for this step, call `report_to_owner` with your results summary and then IMMEDIATELY output your checkpoint completion JSON to advance to the next step. \
                                 If the current step is the final step and the global goal is satisfied, set status to 'completed' in your checkpoint JSON.",
                                task_type_upper
                            ),
                            ref_id: "".to_string(),
                        });
                    }
                }
            }
            // ------------------------------------------------

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
                "AUTONOMOUS_TASK" => format!(
                    "autonomous task: '{}'",
                    params
                        .payload
                        .get("task_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No title")
                ),
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
                Ok(row) => {
                    // Get user name for personalized response
                    let display_name: String = sqlx::query_scalar(
                        "SELECT COALESCE(display_name, 'Human') FROM users WHERE id = $1",
                    )
                    .bind(user_id)
                    .fetch_one(&dispatcher.pool)
                    .await
                    .unwrap_or_else(|_| "Human".to_string());

                    let content = format!(
                        "Got it, {}! I've scheduled your {} for {}.",
                        display_name,
                        task_description,
                        due_at_wib.format("%H:%M WIB").to_string()
                    );

                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content,
                        follow_up_prompt: "".to_string(),
                        ref_id: row.id.to_string(),
                    })
                }
                Err(e) => Ok(ToolResult {
                    error: format!("Failed to schedule task: {}", e),
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
