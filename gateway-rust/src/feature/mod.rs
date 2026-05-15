use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub mod admin;
pub mod conversation;
pub mod graph;
pub mod message_processor;
pub mod realtime;
pub mod redis;
pub mod waitlist;

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
