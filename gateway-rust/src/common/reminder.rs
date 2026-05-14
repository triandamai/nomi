use crate::AppState;
use crate::common::repository::message_repo::save_message;
use crate::feature::OutboundMessage;
use chrono::{DateTime, Duration, Months, TimeZone, Utc};
use chrono_tz::Tz;
use serde_json::json;
use tracing::{error, info};

pub async fn start_reminder_worker(state: AppState) {
    info!("Starting reminder background worker...");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Err(e) = process_reminders(&state).await {
            error!("Error processing reminders: {}", e);
        }
    }
}

async fn process_reminders(state: &AppState) -> anyhow::Result<()> {
    // 1. Fetch pending tasks that are due
    let due_tasks = sqlx::query!(
        r#"
        SELECT r.id, r.user_id, r.conversation_id, r.task_type, r.payload, r.due_at, 
               r.frequency, r.interval_count, r.max_repeats, r.current_runs
        FROM reminders r
        WHERE r.status = 'pending' AND r.due_at <= NOW()
        LIMIT 20
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    for task in due_tasks {
        let task_type = task.task_type.as_deref().unwrap_or("REMINDER");
        info!("Processing due task: {} of type {}", task.id, task_type);

        match task_type {
            "REMINDER" => {
                if let Some(payload) = task.payload {
                    let message = payload
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No content");

                    if let Some(conversation_id) = task.conversation_id {
                        let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
                        let due_local = task.due_at.with_timezone(&tz);
                        let outbound_text = format!(
                            "⏰ *REMINDER ({})*\n{}\n\n_Reply 'done' to complete or 'snooze' to delay._\n(Ref: {})",
                            due_local.format("%H:%M"),
                            message,
                            task.id
                        );

                        if let Ok(m) = save_message(
                            &state.pool,
                            conversation_id,
                            "assistant",
                            outbound_text.as_str(),
                            None,
                            Some(task.user_id),
                            0,
                            0,
                            0,
                            None,
                            None,
                            None,
                            None,
                            None,
                        )
                        .await
                        {
                            let _ = state
                                .send_sse_to_user(
                                    task.user_id.to_string().as_str(),
                                    "message",
                                    json!({
                                        "id": m.id,
                                        "conversation_id": conversation_id,
                                        "role": m.role,
                                        "content": m.content,
                                        "thought": m.thought,
                                        "user_id": m.user_id,
                                        "created_at": m.created_at,
                                    }),
                                )
                                .await;
                        }

                        let channels = sqlx::query!(
                        "SELECT channel_type, external_chat_id, external_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC",
                        task.user_id
                    ).fetch_all(&state.pool).await.unwrap_or_default();

                        for channel in channels {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: channel.external_id,
                                conversation_id: channel.external_chat_id,
                                text: outbound_text.clone(),
                                channel: channel.channel_type,
                                video_url: None,
                                image_url: None,
                                audio_url: None,
                                doc_url: None,
                                sticker_url: None,
                                metadata: Some(json!({
                                    "reminder_id": task.id,
                                    "type": "reminder"
                                })),
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            }
            "SEND_DM" => {
                if let Some(payload) = task.payload {
                    let recipient_jid = payload.get("recipient_jid").and_then(|v| v.as_str());
                    let message = payload.get("message").and_then(|v| v.as_str());

                    if let (Some(jid), Some(msg)) = (recipient_jid, message) {
                        let channel = sqlx::query!(
                            "SELECT channel_type, external_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
                            task.user_id
                        ).fetch_one(&state.pool).await;

                        if let Ok(channel) = channel {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: channel.external_id,
                                conversation_id: jid.to_string(),
                                text: msg.to_string(),
                                channel: channel.channel_type,
                                video_url: None,
                                image_url: None,
                                audio_url: None,
                                doc_url: None,
                                sticker_url: None,
                                metadata: Some(json!({
                                    "task_id": task.id,
                                    "type": "automated_dm"
                                })),
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            }
            "TRIGGER_AGENT" => {
                info!(
                    "Spinning up isolated background execution pool for task {}",
                    task.id
                );
                // Placeholder for background agent execution logic
            }
            _ => error!("Unknown task type: {}", task_type),
        }

        let next_run = task.current_runs.unwrap_or(0) + 1;
        let freq = task.frequency.as_deref().unwrap_or("once");

        let is_done = if let Some(max) = task.max_repeats {
            next_run >= max
        } else {
            freq == "once"
        };

        if is_done {
            sqlx::query!(
                "UPDATE reminders SET status = 'completed', current_runs = $1, updated_at = NOW() WHERE id = $2",
                next_run,
                task.id
            )
            .execute(&state.pool)
            .await?;
        } else {
            let next_due = calculate_next_due(task.due_at, freq);
            sqlx::query!(
                "UPDATE reminders SET due_at = $1, current_runs = $2, updated_at = NOW() WHERE id = $3",
                next_due,
                next_run,
                task.id
            )
            .execute(&state.pool)
            .await?;
        }
    }

    Ok(())
}

fn calculate_next_due(current: DateTime<Utc>, frequency: &str) -> DateTime<Utc> {
    match frequency {
        "daily" => current + Duration::days(1),
        "weekly" => current + Duration::weeks(1),
        "monthly" => current + Months::new(1),
        _ => current, // Should not happen for recurring
    }
}

pub async fn handle_get_reminders(
    axum::extract::State(state): axum::extract::State<AppState>,
    axum::extract::Extension(claims): axum::extract::Extension<
        crate::feature::conversation::auth::Claims,
    >,
) -> crate::common::api_response::ApiResponse<
    Vec<crate::feature::conversation::model::ReminderResponse>,
> {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return crate::common::api_response::ApiResponse::failed("Invalid user ID in token");
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT 
            id,
            task_type as "task_type!",
            payload as "payload!",
            COALESCE(payload->>'message', content) as "content!",
            (due_at AT TIME ZONE 'Asia/Jakarta') as due_at,
            frequency,
            status,
            created_at
        FROM reminders
        WHERE user_id = $1
        ORDER BY due_at ASC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(rows) => {
            let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
            let reminders = rows
                .into_iter()
                .map(|r| crate::feature::conversation::model::ReminderResponse {
                    id: r.id,
                    task_type: r.task_type,
                    payload: r.payload,
                    content: r.content,
                    // Convert NaiveDateTime (from AT TIME ZONE) back to DateTime<Utc>
                    // We use the timezone to correctly interpret the naive datetime as Jakarta time, then convert to UTC
                    due_at: tz
                        .from_local_datetime(&r.due_at.unwrap())
                        .single()
                        .unwrap()
                        .with_timezone(&Utc),
                    frequency: r.frequency,
                    status: r.status.unwrap_or_default(),
                    created_at: r.created_at.unwrap_or_else(Utc::now),
                })
                .collect();
            crate::common::api_response::ApiResponse::ok(reminders, "Tasks retrieved")
        }
        Err(e) => {
            error!("Failed to fetch reminders: {}", e);
            crate::common::api_response::ApiResponse::failed("Failed to fetch reminders")
        }
    }
}
