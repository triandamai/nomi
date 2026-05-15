use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub user_id: Option<Uuid>,
    pub text_content: String,
    pub image_url: Option<String>,
    pub audio_url: Option<String>,
    pub video_url: Option<String>,
    pub sticker_url: Option<String>,
    pub doc_url: Option<String>,
    pub source: MessageSource,
    pub v2: bool,
}
