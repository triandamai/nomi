use crate::common::tools::tools_model::ToolResult;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub enum PromptActor {
    #[serde(rename = "user_prompt")]
    User {
        history: String,
        memories: String,
        message: String,
        system_prompt: String,
    },
    #[serde(rename = "multi_tool_prompt")]
    MultiTool {
        history: String,
        memories: String,
        message: String,
        system_prompt: String,
        tool_results: Vec<(String, ToolResult)>, // (tool_name, result)
        previous_calls: Vec<gemini_rust::FunctionCall>,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub thought: String,
    pub code: String,
    pub response: String,
}
