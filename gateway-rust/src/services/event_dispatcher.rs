use crate::common::app_state::AppState;
use crate::feature::{OutboundMessage, PresenceMessage};
use rumqttc::QoS;
use serde_json::Value;
use uuid::Uuid;

pub enum EventScope {
    User(String),
    Conversation(Uuid),
    Broadcast,
}

pub struct AppEvent {
    pub scope: EventScope,
    pub name: String,
    pub payload: Value,
    pub redis_outbound: Option<OutboundMessage>,
    pub redis_presence: Option<PresenceMessage>,
}

impl AppEvent {
    pub fn user(user_id: &str, name: &str, payload: Value) -> Self {
        Self {
            scope: EventScope::User(user_id.to_string()),
            name: name.to_string(),
            payload,
            redis_outbound: None,
            redis_presence: None,
        }
    }

    pub fn conversation(conversation_id: Uuid, name: &str, payload: Value) -> Self {
        Self {
            scope: EventScope::Conversation(conversation_id),
            name: name.to_string(),
            payload,
            redis_outbound: None,
            redis_presence: None,
        }
    }

    pub fn broadcast(name: &str, payload: Value) -> Self {
        Self {
            scope: EventScope::Broadcast,
            name: name.to_string(),
            payload,
            redis_outbound: None,
            redis_presence: None,
        }
    }

    pub fn with_redis_outbound(mut self, msg: OutboundMessage) -> Self {
        self.redis_outbound = Some(msg);
        self
    }

    pub fn with_redis_presence(mut self, msg: PresenceMessage) -> Self {
        self.redis_presence = Some(msg);
        self
    }
}

pub async fn dispatch(state: &AppState, event: AppEvent) -> anyhow::Result<()> {
    let payload_str = event.payload.to_string();

    // 1. Redis Dispatch (Internal Sync)
    if let Some(msg) = &event.redis_outbound {
        let _ = state.publish_outbond(msg).await;
    }
    if let Some(msg) = &event.redis_presence {
        let _ = state.publish_presence(msg).await;
    }

    // 2. MQTT Dispatch
    match event.scope {
        EventScope::User(user_id) => {
            let topic = format!("nomi/users/{}/{}", user_id, event.name);
            if let Err(e) = state.mqtt.publish_event(&topic, &payload_str, QoS::AtLeastOnce).await {
                tracing::warn!("MQTT publish failed [{}]: {:?}", topic, e);
            }
        }
        EventScope::Conversation(conv_id) => {
            let topic = format!("nomi/conversations/{}/{}", conv_id, event.name);
            // task_update uses retain=true: TaskCard components that mount *after* the publish
            // still receive the latest task state immediately from the broker.
            let result = if event.name == "task_update" {
                state.mqtt.publish_retained(&topic, &payload_str, QoS::AtLeastOnce).await
            } else {
                state.mqtt.publish_event(&topic, &payload_str, QoS::AtLeastOnce).await
            };
            if let Err(e) = result {
                tracing::warn!("MQTT publish failed [{}]: {:?}", topic, e);
            }
        }
        EventScope::Broadcast => {
            let topic = format!("nomi/broadcast/{}", event.name);
            if let Err(e) = state.mqtt.publish_event(&topic, &payload_str, QoS::AtLeastOnce).await {
                tracing::warn!("MQTT publish failed [{}]: {:?}", topic, e);
            }
        }
    }

    Ok(())
}
