use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadWorkSpaceParameters {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadWorkSpaceResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteReadQueryParameters {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteReadQueryResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseToJsonParameters {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseToJsonResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetrieveKnowledgeParameters {
    pub query: String,
    pub start_date: Option<String>, // ISO 8601 string
    pub end_date: Option<String>,   // ISO 8601 string
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetrieveKnowledgeResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolResult {
    pub error: String,
    pub success: bool,
    pub content: String,
    pub follow_up_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchWebParameters {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchWebResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadWebPageParameters {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReadWebPageResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConversationSoulParameters {
    pub new_soul: String,
    pub reason_for_change: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConversationSoulResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateKnowledgeBaseParameters {
    pub content: String,
    pub category: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateKnowledgeBaseResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvolveBootstrapParameters {
    pub updated_instructions: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvolveBootstrapResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScheduleTaskParameters {
    pub task_type: String,     // "REMINDER", "SEND_DM", "TRIGGER_AGENT"
    pub due_at: String,        // ISO 8601 absolute timestamp string
    pub payload: serde_json::Value, // Flexible execution variables
    pub frequency: Option<String>,
    pub max_repeats: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScheduleTaskResponse {
    pub task_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModifyReminderParameters {
    pub reminder_id: String,
    pub action: String, // 'snooze', 'cancel', 'done'
    pub snooze_until: Option<String>, // ISO 8601 string if action is 'snooze'
    pub timezone: Option<String>, // Optional timezone
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModifyReminderResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetReminderStatsParameters {
    pub start_after: Option<String>,
    pub end_before: Option<String>,
    pub status_filter: Option<String>,
    pub limit: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetReminderStatsResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchUsersParameters {
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchUsersResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserProfileParameters {
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateUserProfileResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetInboxSummaryParameters {
    pub limit: Option<i32>,
    pub only_strangers: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetInboxSummaryResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendDirectMessageParameters {
    pub recipient_jid: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendDirectMessageResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MakeStickerParameters {
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MakeStickerResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogExpenseParameters {
    pub merchant: String,
    pub total: Option<f64>,
    pub category: String,
    pub items: Vec<LogExpenseItem>,
    pub tax: Option<f64>,
    pub service: Option<f64>,
    pub discount: Option<f64>,
    pub image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogExpenseItem {
    pub name: String,
    pub quantity: i32,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetLatestMediaContextParameters {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetLatestMediaContextResponse {
    pub media_url: Option<String>,
    pub media_type: Option<String>,
    pub classification: Option<String>,
    pub created_at: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeMediaParameters {
    pub prompt: String,
    pub media_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeMediaResponse {
    pub content: String,
    pub prompt_tokens: Option<i32>,
    pub candidates_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogExpenseResponse {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetExpenseSummaryParameters {
    pub period: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetExpenseSummaryResponse {
    pub total_expenses: f64,
    pub total_income: f64,
    pub top_category: Option<String>,
    pub trend_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTransactionDetailsParameters {
    pub date: Option<String>,     // ISO 8601 string, defaults to today
    pub category: Option<String>, // Optional category filter
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetTransactionDetailsResponse {
    pub transactions: Vec<TransactionDetail>,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionDetail {
    pub merchant_name: Option<String>,
    pub total_amount: f64,
    pub category: Option<String>,
    pub description: Option<String>,
    pub items: Vec<TransactionItem>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TransactionItem {
    pub name: String,
    pub quantity: i32,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConversationTitleParameters {
    pub new_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateConversationTitleResponse {
    pub updated_title: String,
}
