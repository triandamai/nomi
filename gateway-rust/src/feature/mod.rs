use serde::{Deserialize, Serialize};
use std::fmt::Display;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

pub mod admin;
pub mod conversation;
pub mod graph;
pub mod message_processor;
pub mod redis;
pub mod waitlist;
pub mod health_tracking;
pub mod money_tracking;
pub mod reminder;
pub mod edge_functions;
pub mod diagnostics;
pub mod friendship;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuotedMessage {
    pub message_id: String,
    pub sender_id: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InboundMessage {
    pub is_group: bool,
    pub is_private: bool,
    pub is_mentioned: bool,
    pub sender_id: String,
    pub conversation_id: String,
    pub message_id: String,
    pub text: String,
    pub channel: String,
    pub video_url: Option<String>,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub doc_url: Option<String>,
    pub sticker_url: Option<String>,
    pub quoted_message: Option<QuotedMessage>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub original_meta: Option<serde_json::Value>,
}

impl Display for InboundMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FallBackPayload {
    pub payload: Option<OutboundMessage>,
    pub error: Option<String>,
    pub code: i32,
}

impl Display for FallBackPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutboundMessage {
    pub is_group: bool,
    pub sender_id: String,
    pub conversation_id: String,
    pub text: String,
    pub channel: String,
    pub video_url: Option<String>,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub doc_url: Option<String>,
    pub sticker_url: Option<String>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}
impl Display for OutboundMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresenceMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub channel: String,
    pub status: String, // "typing", "idle"
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    Web { name: String },
    Telegram { name: String },
    WhatsApp { name: String },
    Other { name: String },
    Multiple { source: Vec<String> }
}

#[derive(Debug, Clone)]
pub struct UnifiedMessage {
    pub is_group: bool,
    pub is_mentioned: bool,
    pub conversation_id: Uuid,
    pub display_name: Option<String>,
    pub user_id: Option<Uuid>,
    pub text_content: String,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub video_url: Option<String>,
    pub sticker_url: Option<String>,
    pub doc_url: Option<String>,
    pub source: MessageSource,
    pub quoted_message: Option<QuotedMessage>,
    pub reply_to_id: Option<Uuid>,
    pub v2: bool,
}


#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Conversation {
    pub id: Uuid,
    pub session_id: Option<Uuid>,
    pub title: Option<String>,
    pub soul_content: Option<String>,
    pub bootstrap_content: Option<String>,
    pub gateway_thresholds: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Display for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}
