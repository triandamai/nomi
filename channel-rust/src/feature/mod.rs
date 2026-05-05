use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod redis;
pub mod conversation;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InboundMessage {
    pub external_id: String, // WA JID or Telegram ID
    pub platform: String,    // "whatsapp" or "telegram"
    pub display_name: Option<String>,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutboundMessage {
    pub external_id: String,
    pub platform: String,
    pub content: String,
    pub thought: Option<String>,
    pub conversation_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresenceMessage {
    pub external_id: String,
    pub platform: String,
    pub status: String, // "typing", "idle"
}
