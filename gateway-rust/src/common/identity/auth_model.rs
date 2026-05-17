use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OtpRequest {
    #[validate(length(min = 1))]
    pub external_id: String, // email, phone, or telegram_id
    pub channel: String,      // "email", "telegram", etc.
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct OtpVerify {
    pub external_id: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub user_id: String,
    pub profile: Option<UserProfile>,
    pub channels: Option<Vec<crate::feature::conversation::model::ChannelStatus>>,
    pub conversations: Option<Vec<crate::feature::conversation::model::ConversationResponse>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub role: Option<String>,
}
