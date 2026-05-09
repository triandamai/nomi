use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSource {
    Web,
    Telegram,
    WhatsApp,
    Other(String),
}

#[derive(Debug, Clone)]
pub struct UnifiedMessage {
    pub conversation_id: Uuid,
    pub user_id: Option<Uuid>,
    pub text_content: String,
    pub image_url: Option<String>,
    pub source: MessageSource,
    pub v2:bool
}
