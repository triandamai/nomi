use crate::common;
use crate::feature::{ OutboundMessage, PresenceMessage};
use gemini_rust::Gemini;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
    pub gemini: Arc<Gemini>,
    pub gemini_api_key: String,
    // pub presence: Arc<PresenceManager>,
    pub redis: common::redis::RedisClient,
    pub storage: common::storage::StorageClient,
    pub model_info: crate::common::agent::agent_model::ModelInfo,
    pub mqtt: Arc<crate::services::mqtt_service::MqttManager>,
}

impl AppState {
    pub async fn send_sse_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/users/{}/{}", user_id, event_name);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        Ok(())
    }
    pub async fn send_to_user(
        &self,
        user_id: &str,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/users/{}/{}", user_id, event_name);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        let _ = self.publish_outbond(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_sse(
        &self,
        event_name: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/broadcast/{}", event_name);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        Ok(())
    }

    pub async fn broadcast_sse_token_update(
        &self,
        conversation_id: &Uuid,
        cumulative_tokens: &u64,
    ) -> anyhow::Result<()> {
        let sse_data = json!({
            "conversation_id": conversation_id,
            "cumulative_tokens": cumulative_tokens
        });

        // Publish to MQTT
        let topic = format!("nomi/conversations/{}/token_update", conversation_id);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        Ok(())
    }

    pub async fn broadcast(
        &self,
        event_name: &str,
        sse_data: serde_json::Value,
        redis_data: &OutboundMessage,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/broadcast/{}", event_name);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        let _ = self.publish_outbond(redis_data).await;
        Ok(())
    }

    pub async fn send_presence_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/users/{}/presence", user_id);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        let _ = self.publish_presence(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_presence(
        &self,
        sse_data: serde_json::Value,
        redis_data: &PresenceMessage,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = "nomi/broadcast/presence".to_string();
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;

        let _ = self.publish_presence(redis_data).await;
        Ok(())
    }

    pub async fn broadcast_presence_sse(&self, sse_data: serde_json::Value) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = "nomi/broadcast/presence".to_string();
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;
        Ok(())
    }
    pub async fn send_presence_sse_to_user(
        &self,
        user_id: &str,
        sse_data: serde_json::Value,
    ) -> anyhow::Result<()> {
        // Publish to MQTT
        let topic = format!("nomi/users/{}/presence", user_id);
        let _ = self.mqtt.publish_event(&topic, &sse_data.to_string(), rumqttc::QoS::AtLeastOnce).await;
        Ok(())
    }

    pub async fn dispatch(&self, event: crate::services::event_dispatcher::AppEvent) -> anyhow::Result<()> {
        crate::services::event_dispatcher::dispatch(self, event).await
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
