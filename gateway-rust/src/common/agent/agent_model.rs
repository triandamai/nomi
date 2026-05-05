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
    #[serde(rename = "tool_prompt")]
    Tool {
        history: String,
        memories: String,
        tool_name: String,
        tool_result: ToolResult,
        message: String,
        system_prompt: String,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub thought: String,
    pub code: String,
    pub response: String,
}
