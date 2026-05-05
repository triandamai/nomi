use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::encode::IsNull::No;
use tracing::{error, info};
use uuid::Uuid;

use crate::AppState;
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

#[derive(Deserialize, Debug)]
pub struct ChatRequest {
    pub conversation_id: Uuid,
    pub message: String,
}

#[derive(Deserialize)]
pub struct CreateConversationRequest {
    pub session_id: Option<Uuid>,
    pub title: Option<String>,
    pub name: Option<String>, // Frontend uses 'name'
    pub soul_content: Option<String>,
    pub bootstrap_content: Option<String>,
}

#[derive(Serialize)]
pub struct ConversationResponse {
    pub id: Uuid,
    pub name: String,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct UpdateConversationRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RestoreSoulRequest {
    pub version: i32,
}

#[derive(Serialize)]
pub struct RestoreSoulResponse {
    pub conversation_id: Uuid,
    pub version: i32,
    pub soul_content: String,
}

#[derive(Serialize)]
pub struct SoulHistoryResponse {
    pub id: Uuid,
    pub version: i32,
    pub change_reason: String,
    pub soul_content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageItem {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub thought: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatStreamChunk {
    pub content: String,    // The main message
    pub thought: String,    // Content from <thinking>
    pub code_block: String, // Content from ```
    pub tool_call: Option<Value>,
}
