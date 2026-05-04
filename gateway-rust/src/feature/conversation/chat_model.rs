use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::encode::IsNull::No;
use tracing::{error, info};
use uuid::Uuid;

use crate::common::agent::agent_model::PromptActor;
use crate::common::agent::{function_call, send_prompt};
use crate::common::api_response::ApiResponse;
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::common::tools::tools_model::{
    ExecuteReadQueryParameters, ReadWorkSpaceParameters, ToolResult,
};
use crate::common::tools::{ArtaTool, ToolDispatcher};
use crate::rag;
use crate::AppState;

#[derive(Deserialize, Debug)]
pub struct ChatRequest {
    pub conversation_id: Uuid,
    pub message: String,
}


#[derive(Deserialize)]
pub struct CreateConversationRequest {
    pub session_id: Option<Uuid>,
    pub title: Option<String>,
    pub soul_content: Option<String>,
    pub bootstrap_content: Option<String>,
}

#[derive(Serialize)]
pub struct CreateConversationResponse {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageItem {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub thought:Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct MessageListParams {
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct MessageListResponse {
    pub messages: Vec<MessageItem>,
    pub next_cursor: Option<DateTime<Utc>>,
}



#[derive(Serialize,Deserialize,Clone,Debug)]
pub struct ChatStreamChunk {
    pub content: String,      // The main message
    pub thought: String,      // Content from <thinking>
    pub code_block: String,   // Content from ```
    pub tool_call: Option<Value>,
}



