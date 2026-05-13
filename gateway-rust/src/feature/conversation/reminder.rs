use crate::AppState;
use chrono::{DateTime, Duration, Months, Utc, TimeZone};
use chrono_tz::Tz;
use crate::common::repository::message_repo::save_message;
use crate::feature::OutboundMessage;
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
    // 1. Fetch pending reminders that are due
    // We join with channels to know where to send the message.
    // If conversation_id is present, we try to use the channel associated with it.
    let due_reminders = sqlx::query!(
        r#"
        SELECT r.id, r.user_id, r.conversation_id, r.content, r.due_at, 
               r.frequency, r.interval_count, r.max_repeats, r.current_runs
        FROM reminders r
        WHERE r.status = 'pending' AND r.due_at <= NOW()
        LIMIT 20
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    for reminder in due_reminders {
        info!("Processing due reminder: {}", reminder.id);
        // 2. Determine target channel

        // Fallback: try to find the most recent channel for this user
        let channels = sqlx::query!(
                "SELECT channel_type, external_chat_id, external_id,conversation_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
                reminder.user_id
            ).fetch_all(&state.pool).await?;

        if channels.len() < 1 {
            error!(
                "No channel found for user {} to send reminder {}",
                reminder.user_id, reminder.id
            );
            sqlx::query!(
                "UPDATE reminders SET status = 'error', updated_at = NOW() WHERE id = $1",
                reminder.id
            )
            .execute(&state.pool)
            .await?;
            continue;
        }

        match reminder.conversation_id {
            Some(conversation_id) => {
                info!("saving reminder message");
                let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
                let due_local = reminder.due_at.with_timezone(&tz);
                let outbound_text = format!(
                    "⏰ *REMINDER ({})*
{}

_Reply 'done' to complete or 'snooze' to delay._
(Ref: {})",
                    due_local.format("%H:%M"),
                    reminder.content, reminder.id
                );

                if let Ok(m) = save_message(
                    &state.pool,
                    conversation_id,
                    "assistant",
                    outbound_text.clone().as_str(),
                    None,
                    Some(reminder.user_id),
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
                            reminder.user_id.to_string().as_str(),
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

                for channel in channels {
                    let outbound = OutboundMessage {
                        is_group: false,
                        sender_id: channel.external_id.clone(),
                        conversation_id: channel.external_chat_id.clone(),
                        text: outbound_text.clone(),
                        channel: channel.channel_type.clone(),
                        video_url: None,
                        image_url: None,
                        audio_url: None,
                        doc_url: None,
                        sticker_url: None,
                        metadata: Some(json!({
                                "reminder_id": reminder.id,
                                "type": "reminder"
                        })),
                    };
                    info!("Sending reminder: {:?}", outbound);
                    let _ = state.publish_outbond(&outbound).await;
                }
            }
            None => {
                error!(
                    "No conversation found for user {} to send reminder",
                    reminder.user_id
                );
            }
        }

        let next_run = reminder.current_runs.unwrap_or(0) + 1;
        let freq = reminder.frequency.as_deref().unwrap_or("once");

        let is_done = if let Some(max) = reminder.max_repeats {
            next_run >= max
        } else {
            freq == "once"
        };

        if is_done {
            sqlx::query!(
                "UPDATE reminders SET status = 'completed', current_runs = $1, updated_at = NOW() WHERE id = $2",
                next_run,
                reminder.id
            )
            .execute(&state.pool)
            .await?;
        } else {
            let next_due = calculate_next_due(reminder.due_at, freq);
            sqlx::query!(
                "UPDATE reminders SET due_at = $1, current_runs = $2, updated_at = NOW() WHERE id = $3",
                next_due,
                next_run,
                reminder.id
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
    Vec<crate::feature::conversation::chat_model::ReminderResponse>,
> {
    let user_id = match uuid::Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return crate::common::api_response::ApiResponse::failed("Invalid user ID in token");
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT id, content, (due_at AT TIME ZONE 'Asia/Jakarta') as due_at, frequency, status, created_at
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
                .map(
                    |r| crate::feature::conversation::chat_model::ReminderResponse {
                        id: r.id,
                        content: r.content,
                        // Convert NaiveDateTime (from AT TIME ZONE) back to DateTime<Utc>
                        // We use the timezone to correctly interpret the naive datetime as Jakarta time, then convert to UTC
                        due_at: tz.from_local_datetime(&r.due_at.unwrap()).single().unwrap().with_timezone(&Utc),
                        frequency: r.frequency,
                        status: r.status.unwrap_or_default(),
                        created_at: r.created_at.unwrap_or_else(Utc::now),
                    },
                )
                .collect();
            crate::common::api_response::ApiResponse::ok(reminders, "Reminders retrieved")
        }
        Err(e) => {
            error!("Failed to fetch reminders: {}", e);
            crate::common::api_response::ApiResponse::failed("Failed to fetch reminders")
        }
    }
}
