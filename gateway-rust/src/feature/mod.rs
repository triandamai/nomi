use serde::{Deserialize, Serialize};

pub mod redis;
pub mod conversation;
pub mod realtime;
pub mod graph;
pub mod message_processor;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InboundMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutboundMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PresenceMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub channel: String,
    pub status: String, // "typing", "idle"
}
