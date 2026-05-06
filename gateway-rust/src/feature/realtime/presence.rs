use crate::AppState;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{Duration, Instant};
use tracing::{error, info};
use uuid::Uuid;

pub struct PresenceManager {
    // conversation_id -> last_activity
    pub debouncers: DashMap<Uuid, Instant>,
    pub channel_tx: mpsc::Sender<DebounceEvent>,
}

#[derive(Debug)]
pub enum DebounceEvent {
    NewMessage(Uuid, Uuid), // conversation_id, user_id
}

impl PresenceManager {
    pub fn new(state: AppState) -> Arc<Self> {
        let (tx, mut rx) = mpsc::channel::<DebounceEvent>(100);
        let manager = Arc::new(Self {
            debouncers: DashMap::new(),
            channel_tx: tx,
        });

        let manager_clone = manager.clone();
        tokio::spawn(async move {
            let pending_notifications: DashMap<Uuid, (Uuid, Instant)> = DashMap::new();

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        match event {
                            DebounceEvent::NewMessage(conv_id, user_id) => {
                                pending_notifications.insert(conv_id, (user_id, Instant::now() + Duration::from_secs(2)));
                            }
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {
                        let now = Instant::now();
                        let mut to_process = Vec::new();

                        for entry in pending_notifications.iter() {
                            if now >= entry.value().1 {
                                to_process.push(*entry.key());
                            }
                        }

                        for conv_id in to_process {
                            if let Some((_, (user_id, _))) = pending_notifications.remove(&conv_id) {
                                let state_inner = state.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = process_debounced_message(state_inner, conv_id, user_id).await {
                                        error!("Error processing debounced message: {}", e);
                                    }
                                });
                            }
                        }
                    }
                }
            }
        });

        manager_clone
    }
}

async fn process_debounced_message(
    state: AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<()> {
    info!(conversation_id = %conversation_id, "Processing debounced message for user {}", user_id);

    // 1. Fetch channel info to determine source
    let channel_info = sqlx::query!(
        "SELECT channel_type FROM channels WHERE user_id = $1 AND conversation_id = $2",
        user_id,
        conversation_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let source = match channel_info {
        Some(c) => match c.channel_type.as_str() {
            "telegram" => crate::feature::message_processor::MessageSource::Telegram,
            "whatsapp" => crate::feature::message_processor::MessageSource::WhatsApp,
            other => crate::feature::message_processor::MessageSource::Other(other.to_string()),
        },
        None => crate::feature::message_processor::MessageSource::Web, // Fallback, though usually it's a channel if it's debounced
    };

    // 2. Fetch the most recent messages that haven't been responded to yet
    let last_messages = sqlx::query!(
        "SELECT role, content FROM messages WHERE conversation_id = $1 ORDER BY created_at DESC LIMIT 10",
        conversation_id
    ).fetch_all(&state.pool).await?;

    let mut user_content = Vec::new();
    for msg in last_messages {
        if msg.role == "user" {
            user_content.push(msg.content);
        } else {
            break;
        }
    }
    user_content.reverse();

    if user_content.is_empty() {
        info!(
            "No new user messages found for conversation {}, skipping",
            conversation_id
        );
        return Ok(());
    }

    let combined_message = user_content.join("\n");

    let unified_msg = crate::feature::message_processor::UnifiedMessage {
        conversation_id,
        user_id: Some(user_id),
        text_content: combined_message,
        source,
    };

    // 3. Process via unified engine
    crate::feature::message_processor::process_incoming_message(state, unified_msg).await?;

    Ok(())
}
