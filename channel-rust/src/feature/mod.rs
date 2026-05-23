use serde::{Deserialize, Serialize};
pub mod redis;
pub mod telegram;
pub mod whatsapp;
pub mod bridge;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuotedMessage {
    pub message_id: String,
    pub sender_id: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InboundMessage {
    pub is_group: bool,
    pub is_private: bool,
    pub is_mentioned: bool,
    pub sender_id: String,
    pub conversation_id: String,
    pub message_id: String,
    pub text: String,
    pub channel: String,
    pub video_url:Option<String>,
    pub image_url:Option<String>,
    pub audio_url:Option<String>,
    pub doc_url:Option<String>,
    pub sticker_url:Option<String>,
    pub quoted_message: Option<QuotedMessage>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub original_meta: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OutboundMessage {
    pub is_group: bool,
    pub sender_id: Option<String>,
    pub conversation_id: String,
    pub text: String,
    pub channel: String,
    pub video_url:Option<String>,
    pub image_url:Option<String>,
    pub audio_url:Option<String>,
    pub doc_url:Option<String>,
    pub sticker_url:Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FallBackPayload {
    pub payload: Option<OutboundMessage>,
    pub error: Option<String>,
    pub code:i32
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

#[derive(Debug, Clone)]
pub enum WhatsAppCommand {
    Logout,
    GenerateNewQr,
    Restart,
    SendTyping(String, bool)
}
