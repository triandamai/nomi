use crate::AppState;
use crate::services::event_dispatcher::AppEvent;
use crate::common::identity;
use crate::common::repository::message_repo::save_message;
use crate::feature::{Conversation, UnifiedMessage, MessageSource, OutboundMessage};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::FromRow;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TaskData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub content: Option<String>,
    pub task_type: String,
    pub frequency: Option<String>,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub payload: Option<serde_json::Value>,
}

pub async fn start_schedule_worker(state: AppState) {
    info!("Starting Reminder Schedule Worker...");
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

    loop {
        interval.tick().await;

        let now = Utc::now();
        // Use raw query to handle mapping safely
        let tasks = sqlx::query_as::<_, TaskData>(
            r#"
            SELECT id, user_id, conversation_id, content, task_type, frequency, status, due_at, payload
            FROM reminders
            WHERE status = 'pending' AND due_at <= $1
            "#
        )
        .bind(now)
        .fetch_all(&state.pool)
        .await;

        match tasks {
            Ok(tasks) => {
                for task in tasks {
                    info!("Processing scheduled task: {} ({})", task.id, task.task_type);

                    let res = match task.task_type.as_str() {
                        "REMINDER" => handle_reminder_task(&state, &task).await,
                        "SEND_DM" => handle_send_dm_task(&state, &task).await,
                        "TRIGGER_AGENT" => handle_trigger_agent_task(&state, &task).await,
                        _ => {
                            error!("Unknown task type: {}", task.task_type);
                            Ok(())
                        }
                    };

                    if res.is_ok() {
                        let _ = sqlx::query!(
                            "UPDATE reminders SET status = 'completed', updated_at = NOW() WHERE id = $1",
                            task.id
                        )
                        .execute(&state.pool)
                        .await;
                    }
                }
            }
            Err(e) => error!("Error fetching scheduled tasks: {}", e),
        }
    }
}

pub async fn handle_reminder_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
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
                None,
                None,
                Some(&state.redis),
            )
            .await
            {
                let _ = state
                    .dispatch(AppEvent::user(
                        task.user_id.to_string().as_str(),
                        "message",
                        json!(m),
                    ))
                    .await;
            }

            let channels = sqlx::query!(
                "SELECT channel_type, external_chat_id, external_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC",
                task.user_id
            ).fetch_all(&state.pool).await.unwrap_or_default();
            for channel in channels {
                let outbound = OutboundMessage {
                    channel: channel.channel_type.clone(),
                    conversation_id: channel.external_chat_id.clone(),
                    sender_id: channel.external_id.clone(),
                    text: outbound_text.clone(),
                    is_group: false,
                    image_url: None,
                    video_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: Some(json!({"is_mentioned": true})),
                };
                
                let topic = format!("nomi/conversations/{}/outbound", outbound.conversation_id);
                let _ = state.mqtt.publish_event(&topic, &outbound.to_string(), rumqttc::QoS::AtLeastOnce).await;
            }
        }
    }
    Ok(())
}

pub async fn handle_send_dm_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
    if let Some(payload) = &task.payload {
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("No Content");

        if let Some(conversation_id) = task.conversation_id {
            if let Ok(m) = save_message(
                &state.pool,
                conversation_id,
                "assistant",
                message,
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
                None,
                None,
                Some(&state.redis),
            )
            .await
            {
                let _ = state
                    .dispatch(AppEvent::user(
                        task.user_id.to_string().as_str(),
                        "message",
                        json!(m),
                    ))
                    .await;
            }

            let channels = sqlx::query!(
                "SELECT channel_type, external_chat_id, external_id FROM channels WHERE user_id = $1 ORDER BY created_at DESC",
                task.user_id
            ).fetch_all(&state.pool).await.unwrap_or_default();
            for channel in channels {
                let outbound = OutboundMessage {
                    channel: channel.channel_type.clone(),
                    conversation_id: channel.external_chat_id.clone(),
                    sender_id: channel.external_id.clone(),
                    text: message.to_string(),
                    is_group: false,
                    image_url: None,
                    video_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: Some(json!({"is_mentioned": true})),
                };
                let topic = format!("nomi/conversations/{}/outbound", outbound.conversation_id);
                let _ = state.mqtt.publish_event(&topic, &outbound.to_string(), rumqttc::QoS::AtLeastOnce).await;
            }
        }
    }
    Ok(())
}

pub async fn handle_trigger_agent_task(state: &AppState, task: &TaskData) -> anyhow::Result<()> {
    if let Some(payload) = &task.payload {
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Triggered by agent");

        if let Some(conversation_id) = task.conversation_id {
            let user_info = identity::resolve_identity_by_id(&state.pool, task.user_id)
                .await
                .map_err(|e| anyhow::anyhow!("Identity error: {}", e))?;

            // Fetch conversation info
            let conv_info = crate::common::repository::conversation_repo::get_conversation_info(
                &state.pool,
                &state.redis,
                conversation_id,
            ).await?;

            let unified_msg = UnifiedMessage {
                is_group: conv_info.conversation_type != "private",
                is_mentioned: true,
                display_name: Some(user_info.display_name.clone()),
                conversation_id,
                user_id: Some(task.user_id),
                text_content: message.to_string(),
                image_url: None,
                audio_url: None,
                video_url: None,
                sticker_url: None,
                doc_url: None,
                source: MessageSource::Other {
                    name: "agent_scheduler".to_string(),
                },
                quoted_message: None,
                reply_to_id: None,
                v2: true,
            };

            let map_convo = Conversation::from(conv_info);

            tokio::spawn({
                let state = state.clone();
                async move {
                    let _ = crate::feature::message_processor::v2_orchestrator::process_v2_message(
                        state,
                        map_convo,
                        unified_msg,
                    )
                    .await;
                }
            });
        }
    }
    Ok(())
}
