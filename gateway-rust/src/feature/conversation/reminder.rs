use crate::common::repository::message_repo::save_message;
use crate::feature::OutboundMessage;
use crate::AppState;
use chrono::{DateTime, Duration, Months, Utc};
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
               r.frequency, r.interval_count, r.max_repeats, r.current_runs,
               c.channel_type as "channel_type?", c.external_chat_id as "external_chat_id?", c.external_id as "external_id?"
        FROM reminders r
        LEFT JOIN channels c ON (r.conversation_id = c.conversation_id AND r.user_id = c.user_id)
        WHERE r.status = 'pending' AND r.due_at <= NOW()
        LIMIT 20
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    for reminder in due_reminders {
        info!("Processing due reminder: {}", reminder.id);

        // 2. Determine target channel
        let (channel, chat_id, sender_id) = if let (
            Some(ch),
            Some(cid),
            Some(sid),
        ) = (
            reminder.channel_type,
            reminder.external_chat_id,
            reminder.external_id
        ) {
            (ch, cid, sid)
        } else {
            // Fallback: try to find the most recent channel for this user
            let fallback = sqlx::query!(
                "SELECT channel_type, external_chat_id, external_id,conversation_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
                reminder.user_id
            ).fetch_optional(&state.pool).await?;

            if let Some(fb) = fallback {
                (
                    fb.channel_type,
                    fb.external_chat_id,
                    fb.external_id
                )
            } else {
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
        };

        // 3. Construct and publish outbound message
        let outbound_text = format!(
            "⏰ *REMINDER:*\n{}\n\n_Reply 'done' to complete or 'snooze' to delay._\n(Ref: {})",
            reminder.content, reminder.id
        );

        let outbound = OutboundMessage {
            is_group: chat_id.contains("-") || chat_id.contains("@g.us"),
            sender_id: sender_id.clone(),
            chat_id,
            text: outbound_text.clone(),
            channel,
            metadata: Some(serde_json::json!({
                "reminder_id": reminder.id,
                "type": "reminder"
            })),
        };

        info!("Sending reminder: {:?}", outbound);
        match reminder.conversation_id {
            Some(conversation_id) => {
                info!("saving reminder message");
                if let Ok(m) = save_message(
                    &state.pool,
                    conversation_id,
                    "assistant",
                    outbound_text.as_str(),
                    None,
                    Some(reminder.user_id),
                )
                .await
                {
                    let _ = state
                        .send_to_user(
                            sender_id.to_string().as_str(),
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
                            &outbound,
                        )
                        .await;
                }
            }
            None => {
                info!(
                    "No conversation found for user {} to send reminder",
                    sender_id
                );
            }
        }

        // 4. Update recurrence or mark as completed
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
        _ => current + Duration::days(1),
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
        SELECT id, content, due_at, frequency, status, created_at
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
            let reminders = rows
                .into_iter()
                .map(
                    |r| crate::feature::conversation::chat_model::ReminderResponse {
                        id: r.id,
                        content: r.content,
                        due_at: r.due_at,
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
