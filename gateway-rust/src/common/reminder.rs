use crate::AppState;
use crate::common::identity::UserIdentity;
use crate::common::repository::message_repo::save_message;
use crate::feature::OutboundMessage;
use crate::feature::message_processor::v2_agent_orchestrator::V2AgentOrchestrator;
use crate::models::Conversation;
use chrono::{DateTime, Duration, Months, TimeZone, Utc};
use chrono_tz::Tz;
use serde_json::json;
use tracing::{error, info};

pub async fn start_schedule_worker(state: AppState) {
    info!("Starting reminder background worker...");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        if let Err(e) = process_task(&state).await {
            error!("Error processing reminders: {}", e);
        }
    }
}

pub struct TaskData {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub conversation_id: Option<uuid::Uuid>,
    pub task_type: Option<String>,
    pub payload: Option<serde_json::Value>,
    pub due_at: DateTime<Utc>,
    pub frequency: Option<String>,
    pub interval_count: Option<i32>,
    pub max_repeats: Option<i32>,
    pub current_runs: Option<i32>,
}

async fn process_task(state: &AppState) -> anyhow::Result<()> {
    // 1. Fetch pending tasks that are due and claim them to prevent double-processing
    let due_tasks = sqlx::query!(
        r#"
        UPDATE reminders 
        SET status = 'processing', updated_at = NOW() 
        WHERE id IN (
            SELECT id FROM reminders 
            WHERE status = 'pending' AND due_at <= NOW() 
            FOR UPDATE SKIP LOCKED 
            LIMIT 20
        ) 
        RETURNING id, user_id, conversation_id, task_type, payload, due_at, 
                  frequency, interval_count, max_repeats, current_runs
        "#
    )
    .fetch_all(&state.pool)
    .await?;

    for task_row in due_tasks {
        let task = TaskData {
            id: task_row.id,
            user_id: task_row.user_id,
            conversation_id: task_row.conversation_id,
            task_type: task_row.task_type,
            payload: task_row.payload,
            due_at: task_row.due_at,
            frequency: task_row.frequency,
            interval_count: task_row.interval_count,
            max_repeats: task_row.max_repeats,
            current_runs: task_row.current_runs,
        };

        let task_id = task.id;
        let task_type_str = task.task_type.as_deref().unwrap_or("REMINDER");
        info!(
            "Processing claimed task: {} of type {}",
            task_id, task_type_str
        );

        let result = match task_type_str.to_uppercase().as_str() {
            "REMINDER" => handle_reminder_task(state, &task).await,
            "SEND_DM" => handle_send_dm_task(state, &task).await,
            "TRIGGER_AGENT" => handle_trigger_agent_task(state, &task).await,
            _ => {
                let err = format!("Unknown task type: {}", task_type_str);
                error!("{}", err);
                Err(anyhow::anyhow!(err))
            }
        };

        match result {
            Ok(_) => {
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
                        task_id
                    )
                    .execute(&state.pool)
                    .await?;
                } else {
                    let next_due = calculate_next_due(task.due_at, freq);
                    sqlx::query!(
                        "UPDATE reminders SET status = 'pending', due_at = $1, current_runs = $2, updated_at = NOW() WHERE id = $3",
                        next_due,
                        next_run,
                        task_id
                    )
                    .execute(&state.pool)
                    .await?;
                }
            }
            Err(e) => {
                error!("Task {} failed: {}", task_id, e);
                // Temporarily not using error_log until migration is confirmed
                sqlx::query!(
                    "UPDATE reminders SET status = 'failed', updated_at = NOW() WHERE id = $1",
                    task_id
                )
                .execute(&state.pool)
                .await?;
            }
        }
    }

    Ok(())
}

async fn handle_reminder_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
    if let Some(payload) = &task.payload {
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("No Content");

        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or(content);

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
    Ok(())
}

async fn handle_send_dm_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
    if let Some(payload) = &task.payload {
        let recipient_jid = payload.get("recipient_jid").and_then(|v| v.as_str());
        let message = payload.get("message").and_then(|v| v.as_str());

        if let (Some(jid), Some(msg)) = (recipient_jid, message) {
            let channel = sqlx::query!(
                "SELECT channel_type, external_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
                task.user_id
            ).fetch_one(&state.pool).await?;

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
            state.publish_outbond(&outbound).await;
        }
    }
    Ok(())
}

async fn handle_trigger_agent_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
    info!(
        "Spinning up isolated background execution pool for task {}",
        task.id
    );

    let payload = task
        .payload
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing payload for TRIGGER_AGENT"));
    if let Err(err) = payload {
        info!("{:?}", err);
        return Err(anyhow::anyhow!(err));
    }
    let payload = payload?;
    let task_prompt = payload
        .get("task_prompt")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing task_prompt in payload"));

    if let Err(err) = task_prompt {
        info!("{:?}", err);
        return Err(anyhow::anyhow!(err));
    }
    let task_prompt = task_prompt?;

    let conversation_id = task
        .conversation_id
        .ok_or_else(|| anyhow::anyhow!("Missing conversation_id for TRIGGER_AGENT"))?;

    let members = sqlx::query!(
        "SELECT * FROM conversation_members WHERE conversation_id = $1",
        conversation_id
    )
    .fetch_all(&state.pool)
    .await
    .map_or_else(|_| Vec::default(), |v| v)
    .iter()
    .map(|v| v.conversation_id)
    .collect();

    // 1. Initialize V2AgentOrchestrator
    let orchestrator = V2AgentOrchestrator::new(
        state.clone(),
        Some(Conversation {
            id: Default::default(),
            session_id: None,
            title: None,
            soul_content: None,
            bootstrap_content: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        }),
        Some(UserIdentity {
            id: Default::default(),
            display_name: "".to_string(),
        }),
        members,
    );

    // 2. Identify Trigger Type
    let trigger = match payload.get("trigger_type").and_then(|v| v.as_str()) {
        Some("proactive_check") => {
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::ProactiveCheck {
                reason: payload
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Proactive health/status check")
                    .to_string(),
            }
        }
        Some("system_alert") => {
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::SystemAlert {
                reason: payload
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("System notification")
                    .to_string(),
            }
        }
        _ => {
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::UserRequested {
                reason: payload
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("User-scheduled reminder")
                    .to_string(),
            }
        }
    };

    // 3. Execute Background Job
    let final_text = orchestrator
        .process_background_job(task_prompt, trigger)
        .await?;

    // 3. Send results back to conversation
    let tz: Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
    let now_wib = Utc::now().with_timezone(&tz);
    let timestamp = format!("**WIB: {}**", now_wib.format("%Y-%m-%d %H:%M"));

    let outbound_text = format!("{}\n\n{}", timestamp, final_text);

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
                "task_id": task.id,
                "type": "agent_trigger_response"
            })),
        };
        state.publish_outbond(&outbound).await;
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
    axum::extract::Query(params): axum::extract::Query<
        crate::feature::conversation::model::MessageListParams,
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

    let limit = params.limit.unwrap_or(20);
    // Use a very far future date as default cursor for DESC sort
    let cursor = params
        .cursor
        .unwrap_or_else(|| Utc::now() + chrono::Duration::days(365 * 10));

    let result = sqlx::query!(
        r#"
        SELECT 
            r.id,
            r.task_type as "task_type!",
            r.payload as "payload!",
            COALESCE(r.payload->>'message', r.content) as "content!",
            (r.due_at AT TIME ZONE 'Asia/Jakarta') as due_at,
            r.frequency,
            r.status,
            u.display_name as "user_display_name",
            c.title as "conversation_title",
            r.created_at
        FROM reminders r
        LEFT JOIN users u ON r.user_id = u.id
        LEFT JOIN conversations c ON r.conversation_id = c.id
        WHERE r.user_id = $1 AND r.due_at < $2
        ORDER BY r.due_at DESC
        LIMIT $3
        "#,
        user_id,
        cursor,
        limit
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
                    due_at: tz
                        .from_local_datetime(&r.due_at.unwrap())
                        .single()
                        .unwrap()
                        .with_timezone(&Utc),
                    frequency: r.frequency,
                    status: r.status.unwrap_or_default(),
                    user_display_name: r.user_display_name,
                    conversation_title: r.conversation_title,
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
