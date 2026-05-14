use crate::common::tools::tools_model::ToolResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MediaClassification {
    #[serde(rename = "EXPENSE_RECEIPT")]
    ExpenseReceipt,
    #[serde(rename = "MOTORCYCLE_MAINTENANCE")]
    MotorcycleMaintenance,
    #[serde(rename = "TECHNICAL_DOC")]
    TechnicalDoc,
    #[serde(rename = "NATURE")]
    Nature,
    #[serde(rename = "IGNORE")]
    Ignore,
    #[serde(rename = "OTHER")]
    Other,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseData {
    pub merchant: String,
    pub total: f64,
    pub tax: Option<f64>,
    pub service: Option<f64>,
    pub discount: Option<f64>,
    pub items: Vec<ExpenseItem>,
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpenseItem {
    pub name: String,
    pub quantity: i32,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaintenanceData {
    pub part_names: Vec<String>,
    pub service_details: String,
}


#[derive(Deserialize, Debug)]
pub enum PromptActor {
    #[serde(rename = "user_prompt")]
    User {
        history: String,
        memories: String,
        message: String,
        system_prompt: String,
        media: Option<(String, String)>, // (mime_type, base64_data)
    },
    #[serde(rename = "multi_tool_prompt")]
    MultiTool {
        history: String,
        memories: String,
        message: String,
        system_prompt: String,
        /// Turn-based history of tool interactions.
        /// Each turn is a pair of: (All tool calls in that turn, All results for those calls)
        tool_turns: Vec<(Vec<gemini_rust::FunctionCall>, Vec<(String, ToolResult)>)>,
        media: Option<(String, String)>, // (mime_type, base64_data)
    },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub thought: String,
    pub code: String,
    pub response: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub agent_model: String,
    pub rag_embedding: String,
    pub media_classification: String,
    pub media_analyze: String,
}
