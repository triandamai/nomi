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
    pub ref_id: String,
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
    pub task_type: String,     // "REMINDER", "SEND_DM", "TRIGGER_AGENT", "AUTONOMOUS_TASK"
    pub due_at: String,        // ISO 8601 absolute timestamp string
    #[schemars(description = "You must populate it according to these rules:
    1. If task_type is 'REMINDER': Must contain exactly { 'message': 'The reminder text to send to the user' }.
    2. If task_type is 'SEND_DM': Must contain exactly { 'recipient_jid': 'The destination user ID/JID string', 'content': 'The message text to send' }.
    3. If task_type is 'TRIGGER_AGENT': Must contain exactly { 'task_prompt': 'The instruction string for the background routine execution' }.
    4. If task_type is 'AUTONOMOUS_TASK': Must contain exactly { 'task_title': 'Short descriptive title of the task', 'global_goal': 'Detailed target description Nomi must achieve in the background', 'checkpoints': 'An ordered array of step objects representing the sequential checklist plan. Example: [{\"step_index\": 0, \"action_objective\": \"Check weather forecast\", \"status\": \"pending\"}]' }.
    
    CRITICAL: Do not invent or add extra keys outside of these specifications.")]
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
    pub query: Option<String>,
    pub user_id: Option<String>,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
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
    pub total: f64,
    pub category: String,
    pub items: Vec<LogExpenseItem>,
    pub tax: Option<f64>,
    pub service: Option<f64>,
    pub discount: Option<f64>,
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
