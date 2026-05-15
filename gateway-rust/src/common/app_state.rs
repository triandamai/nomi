use crate::common;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::feature::conversation::model::MessageItem;
use crate::feature::{MessageSource, OutboundMessage, PresenceMessage};
use gemini_rust::Gemini;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub sse: Arc<SseBroadcaster>,
    pub pool: Pool<Postgres>,
    pub gemini: Arc<Gemini>,
    pub gemini_api_key: String,
    // pub presence: Arc<PresenceManager>,
    pub redis: common::redis::RedisClient,
    pub storage: common::storage::StorageClient,
    pub model_info: crate::common::agent::agent_model::ModelInfo,
}

impl AppState {
    pub async fn send_sse_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), event_name.to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }
    pub async fn send_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), event_name.to_string()),
                sse_data,
            ))
            .await;
        let _ = self.publish_outbond(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_sse(
        &self,
        event_name: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast(event_name.to_string()),
                sse_data,
            ))
            .await;

        Ok(())
    }

    pub async fn broadcast_sse_token_update(
        &self,
        conversation_id: &Uuid,
        cumulative_tokens: &u64,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast("token_update".to_string()),
                json!({
                    "conversation_id": conversation_id,
                    "cumulative_tokens": cumulative_tokens
                }),
            ))
            .await;

        Ok(())
    }

    pub async fn broadcast(
        &self,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast(event_name.to_string()),
                sse_data,
            ))
            .await;

        let _ = self.publish_outbond(redis_data).await;
        Ok(())
    }

    pub async fn send_presence_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        // info!("sending with sse and publish to subs");
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                sse_data,
            ))
            .await;
        let _ = self.publish_presence(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_presence(
        &self,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast("presence".to_string()),
                sse_data,
            ))
            .await;

        let _ = self.publish_presence(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_presence_sse(&self, sse_data: serde_json::Value) -> anyhow::Result<()> {
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::broadcast("presence".to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }
    pub async fn send_presence_sse_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        let _ = self
            .sse
            .send(SseBuilder::new(
                SseTarget::sent_to_user(user_id.to_string(), "presence".to_string()),
                sse_data,
            ))
            .await;
        Ok(())
    }

    pub async fn publish_outbond(&self, redis_data: &OutboundMessage) {
        match self.redis.publish_event("nomi:outbound", redis_data).await {
            Ok(_) => {
                info!("publish to redis: outbound event sent");
            }
            Err(err) => {
                error!("publish to redis: outbound publishing failed: {}", err);
            }
        };
    }

    pub async fn publish_presence(&self, redis_data: &PresenceMessage) {
        match self.redis.publish_event("nomi:presence", redis_data).await {
            Ok(_) => {
                info!("publish to redis: presence event sent");
            }
            Err(err) => {
                error!("publish to redis: presence publishing failed: {}", err);
            }
        };
    }

    //====================//
    pub fn send_status_update(
        &self,
        members: Vec<Uuid>,
        conversation_id: Uuid,
        source: MessageSource,
        is_group: bool,
        event: String,
        text: String,
    ) {
        info!("send_status_update start");
        let state = self.clone();
        let pool = state.pool.clone();
        let event = event.clone();
        tokio::spawn(async move {
            let convo = sqlx::query!(
                "SELECT conversation_type,id FROM conversations WHERE id = $1",
                conversation_id
            )
            .fetch_one(&pool)
            .await;

            let ch_names = match source.clone() {
                MessageSource::Web { name } => vec![name.to_string()],
                MessageSource::Telegram { name } => vec![name.to_string()],
                MessageSource::WhatsApp { name } => vec![name.to_string()],
                MessageSource::Other { name } => vec![name.to_string()],
                MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
            };

            if let Err(err) = &convo {
                info!("Sent status update failed: {}", err);
            }

            if let Ok(data) = convo {
                if data.conversation_type.eq_ignore_ascii_case("private") {
                    info!("send_status_update web");
                    for member in members {
                        let _ = state
                            .send_sse_to_user(
                                member.to_string().as_str(),
                                event.to_string().as_str(),
                                json!({
                                    "conversation_id": conversation_id,
                                    "text":text
                                }),
                            )
                            .await;
                    }

                    if !is_group {
                        let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                            .fetch_all(&pool)
                            .await
                            .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let outbound = OutboundMessage {
                                is_group,
                                sender_id: channel.external_id.clone(),
                                conversation_id: channel.external_chat_id.clone(),
                                text: text.clone(),
                                channel: channel.channel_type.clone(),
                                video_url: None,
                                image_url: None,
                                audio_url: None,
                                doc_url: None,
                                sticker_url: None,
                                metadata: None,
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                } else {
                    for member in members {
                        let _ = state
                            .send_sse_to_user(
                                member.to_string().as_str(),
                                event.to_string().as_str(),
                                json!({
                                    "conversation_id": conversation_id,
                                    "text":text
                                }),
                            )
                            .await;
                    }

                    if !is_group {
                        info!("send_status_update channel:{:?}", ch_names);
                        let channel_info = sqlx::query!(
                            "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: "".to_string(),
                                conversation_id: channel.external_group_id.clone(),
                                text: text.clone(),
                                channel: channel.channel.clone(),
                                video_url: None,
                                image_url: None,
                                audio_url: None,
                                doc_url: None,
                                sticker_url: None,
                                metadata: None,
                            };
                            let _ = state.publish_outbond(&outbound).await;
                        }
                    }
                }
            }
        });
    }

    pub fn send_status_presence_update(
        &self,
        members: Vec<Uuid>,
        conversation_id: Uuid,
        source: MessageSource,
        is_group: bool,
        is_typing: bool,
    ) {
        info!("send_status_update start");
        let state = self.clone();
        let pool = state.pool.clone();
        let event = "presence".to_string();
        tokio::spawn(async move {
            let convo = sqlx::query!(
                "SELECT conversation_type,id FROM conversations WHERE id = $1",
                conversation_id
            )
            .fetch_one(&pool)
            .await;

            let ch_names = match source.clone() {
                MessageSource::Web { name } => vec![name.to_string()],
                MessageSource::Telegram { name } => vec![name.to_string()],
                MessageSource::WhatsApp { name } => vec![name.to_string()],
                MessageSource::Other { name } => vec![name.to_string()],
                MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
            };

            if let Err(err) = &convo {
                info!("Sent status update failed: {}", err);
            }

            if let Ok(data) = convo {
                if data.conversation_type.eq_ignore_ascii_case("private") {
                    info!("send_status_update web");
                    for member in members {
                        let _ = state
                            .send_sse_to_user(
                                member.to_string().as_str(),
                                event.to_string().as_str(),
                                json!({"conversation_id": conversation_id,"is_typing": is_typing,"user_id": "nomi"}),
                            )
                            .await;
                    }

                    if !is_group {
                        let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                            .fetch_all(&pool)
                            .await
                            .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let presence = PresenceMessage {
                                sender_id: channel.external_id.clone(),
                                chat_id: channel.external_chat_id.clone(),
                                channel: channel.channel_type.clone(),
                                status: "typing".to_string(),
                            };
                            let _ = state.publish_presence(&presence).await;
                        }
                    }
                } else {
                    for member in members {
                        let _ = state
                            .send_sse_to_user(
                                member.to_string().as_str(),
                                event.to_string().as_str(),
                                json!({"conversation_id": conversation_id,"is_typing": is_typing,"user_id": "nomi"}),
                            )
                            .await;
                    }

                    if !is_group {
                        info!("send_status_update channel:{:?}", ch_names);
                        let channel_info = sqlx::query!(
                            "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                        for channel in channel_info {
                            let presence = PresenceMessage {
                                sender_id: channel.external_group_id.clone(),
                                chat_id: channel.external_group_id.clone(),
                                channel: channel.channel.clone(),
                                status: "typing".to_string(),
                            };
                            let _ = state.publish_presence(&presence).await;
                        }
                    }
                }
            }
        });
    }

    pub fn send_message_to_subscriber(
        &self,
        members: Vec<Uuid>,
        conversation_id: Uuid,
        source: MessageSource,
        sse_data: serde_json::Value,
        data: MessageItem,
    ) {
        let state = self.clone();
        let pool = state.pool.clone();
        let outbound_message = data.clone();
        tokio::spawn(async move {
            let convo = sqlx::query!(
                "SELECT conversation_type,id FROM conversations WHERE id = $1",
                conversation_id
            )
            .fetch_one(&pool)
            .await;

            let ch_names = match source.clone() {
                MessageSource::Web { name } => vec![name.to_string()],
                MessageSource::Telegram { name } => vec![name.to_string()],
                MessageSource::WhatsApp { name } => vec![name.to_string()],
                MessageSource::Other { name } => vec![name.to_string()],
                MessageSource::Multiple { source } => source.iter().map(|s| s.clone()).collect(),
            };

            if let Err(err) = &convo {
                info!("Sent status update failed: {}", err);
            }

            if let Ok(convo) = convo {
                for member in members {
                    let _ = state
                        .send_sse_to_user(member.to_string().as_str(), "message", sse_data.clone())
                        .await;
                }

                // --- Multi-bubble Sequential Burst Strategy ---
                let bubbles =
                    crate::common::splitter::split_into_bubbles(&outbound_message.content);

                if convo.conversation_type.eq_ignore_ascii_case("private") {
                    let channel_info = sqlx::query!(
                            "SELECT c.channel_type, c.external_id, c.external_chat_id, cm.user_id
                                    FROM channels c
                                    JOIN conversation_members cm ON c.user_id = cm.user_id
                                    WHERE cm.conversation_id = $1 AND c.channel_type = ANY($2::text[])",
                            conversation_id,
                            &ch_names[..]
                        )
                        .fetch_all(&pool)
                        .await
                        .unwrap_or(Vec::new());

                    for channel in channel_info {
                        for (i, bubble_text) in bubbles.iter().enumerate() {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: channel.external_id.clone(),
                                conversation_id: channel.external_chat_id.clone(),
                                text: bubble_text.clone(),
                                channel: channel.channel_type.clone(),
                                // Attach media only to the first bubble
                                video_url: if i == 0 {
                                    outbound_message.video_url.clone()
                                } else {
                                    None
                                },
                                image_url: if i == 0 {
                                    outbound_message.image_url.clone()
                                } else {
                                    None
                                },
                                audio_url: if i == 0 {
                                    outbound_message.audio_url.clone()
                                } else {
                                    None
                                },
                                doc_url: if i == 0 {
                                    outbound_message.document_url.clone()
                                } else {
                                    None
                                },
                                sticker_url: if i == 0 {
                                    outbound_message.sticker_url.clone()
                                } else {
                                    None
                                },
                                metadata: None,
                            };
                            info!("Sent outbound message: {}", outbound);
                            let _ = state.publish_outbond(&outbound).await;

                            if i < bubbles.len() - 1 {
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                            }
                        }
                    }
                } else {
                    let channel_info = sqlx::query!(
                        "SELECT c.conversation_id, c.channel, c.external_group_id
                            FROM channel_group c
                            WHERE c.conversation_id = $1 AND c.channel =  ANY($2::text[])",
                        conversation_id,
                        &ch_names[..]
                    )
                    .fetch_all(&pool)
                    .await
                    .unwrap_or(Vec::new());

                    for channel in channel_info {
                        for (i, bubble_text) in bubbles.iter().enumerate() {
                            let outbound = OutboundMessage {
                                is_group: false,
                                sender_id: "".to_string(),
                                conversation_id: channel.external_group_id.clone(),
                                text: bubble_text.clone(),
                                channel: channel.channel.clone(),
                                // Attach media only to the first bubble
                                video_url: if i == 0 {
                                    outbound_message.video_url.clone()
                                } else {
                                    None
                                },
                                image_url: if i == 0 {
                                    outbound_message.image_url.clone()
                                } else {
                                    None
                                },
                                audio_url: if i == 0 {
                                    outbound_message.audio_url.clone()
                                } else {
                                    None
                                },
                                doc_url: if i == 0 {
                                    outbound_message.document_url.clone()
                                } else {
                                    None
                                },
                                sticker_url: if i == 0 {
                                    outbound_message.sticker_url.clone()
                                } else {
                                    None
                                },
                                metadata: None,
                            };
                            info!("Sent outbound message: {}", outbound);
                            let _ = state.publish_outbond(&outbound).await;

                            if i < bubbles.len() - 1 {
                                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                            }
                        }
                    }
                }
            }
        });
    }
}
