use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct InboundMessage {
    pub sender_id: String,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InternalOutboundRequest {
    pub sender_id: String,
    pub chat_id: String,
    pub text: String,
    pub channel: String,
}
