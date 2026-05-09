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
        /// Turn-based history of tool interactions.
        /// Each turn is a pair of: (All tool calls in that turn, All results for those calls)
        tool_turns: Vec<(Vec<gemini_rust::FunctionCall>, Vec<(String, ToolResult)>)>,
    },
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatResponse {
    pub thought: String,
    pub code: String,
    pub response: String,
}
