use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};

pub struct GetWorkspaceSummaryPlugin;

impl NomiToolPlugin for GetWorkspaceSummaryPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "get_workspace_summary",
            "description": "INSPECTION TOOL: Call this to retrieve the current conversation's technical state, including total token usage, remaining budget, and the current Dynamic Execution Boundaries (DEB) thresholds (Sociability, Confidence, Vigilance). Useful when the user asks about your 'settings', 'persona', 'usage', or 'limits'.",
            "parameters": {
                "type": "object",
                "properties": {},
                "required": []
            }
        })
    }

    fn rules(&self) -> &str {
        "1. DATA SOURCE: This tool extracts data from the active session's secure cache.\n\
         2. INTERPRETATION GUIDE:\n\
           - Sociability (interaction_gate): Low = Proactive, High = Passive.\n\
           - Confidence (intent_classification): Low = Aggressive Tooling, High = Strict/Conservative.\n\
           - Vigilance (guardrails): Low = Permissive, High = Maximum Security.\n\
         3. FORMATTING: Present the findings naturally. Do not just list decimals; explain what they mean for your current behavior."
    }

    fn matching_intents(&self) -> &[&str] {
        &["GET_SYSTEM_STATS", "GET_USER_INFO", "GET_SETTINGS"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        _args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let cid = match dispatcher.conversation_id {
                Some(id) => id,
                None => return Ok(ToolResult {
                    success: false,
                    content: "".to_string(),
                    error: "No active conversation context found.".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
            };

            // Fetch latest from repository (benefits from Redis cache)
            match crate::common::repository::conversation_repo::get_conversation_info(
                &dispatcher.pool,
                &dispatcher.app_state.redis,
                cid
            ).await {
                Ok(info) => {
                    let summary = json!({
                        "session_id": info.id,
                        "title": info.title,
                        "usage": {
                            "cumulative_tokens": info.cumulative_tokens,
                            "max_token_usage": info.max_token_usage,
                            "percentage_used": format!("{:.2}%", (info.cumulative_tokens as f64 / info.max_token_usage as f64) * 100.0)
                        },
                        "behavior_boundaries": info.gateway_thresholds,
                        "created_at": info.created_at
                    });

                    Ok(ToolResult {
                        success: true,
                        content: format!("WORKSPACE_SUMMARY_SNAPSHOT: {}", summary),
                        error: "".to_string(),
                        follow_up_prompt: "Based on this snapshot, give the user a warm and insightful update on your current operational state and persona configuration. Explain what the DEB values mean for how you are interacting right now.".to_string(),
                        ref_id: format!("deb_{}", cid),
                    })
                },
                Err(e) => {
                    Ok(ToolResult {
                        success: false,
                        content: "".to_string(),
                        error: format!("Failed to retrieve workspace data: {}", e),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
            }
        }
        .boxed()
    }
}
