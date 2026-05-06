use serde::{Deserialize, Serialize};
pub mod conversation;
pub mod redis;
pub mod telegram;
pub mod whatsapp;
pub mod bridge;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InboundMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutboundMessage {
    pub sender_id: Option<String>,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
    pub user_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TypingRequest {
    pub chat_id: String,
    pub channel: String,
    pub is_typing: bool,
    pub user_id: Option<uuid::Uuid>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PresenceMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub channel: String,
    pub status: String,
}
