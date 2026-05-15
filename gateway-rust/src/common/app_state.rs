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
}
