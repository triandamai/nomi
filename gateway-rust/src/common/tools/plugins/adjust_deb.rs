use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};

pub struct AdjustDebPlugin;

impl NomiToolPlugin for AdjustDebPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "adjust_deb",
            "description": "DYNAMIC BOUNDARY CALIBRATION: Call this tool immediately when the user tells you to change your responsiveness, strictness, proactivity, or safety filters. This modifies runtime Dynamic Execution Boundaries (DEB).",
            "parameters": {
                "type": "object",
                "properties": {
                    "target_layer": { 
                        "type": "string", 
                        "enum": ["interaction_gate", "intent_classification", "guardrails"],
                        "description": "The specific boundary filtering layer to alter."
                    },
                    "new_threshold_value": { 
                        "type": "number", 
                        "description": "The newly computed parameter decimal. Must scale between 0.0 and 1.0." 
                    },
                    "explanation": { "type": "string", "description": "Brief summary of calculation parameters and which mode was selected." }
                },
                "required": ["target_layer", "new_threshold_value", "explanation"]
            }
        })
    }

    fn rules(&self) -> &str { 
        "1. MAP MODES TO VALUES:\n\
         - Sociability (interaction_gate):\n\
           * <= 0.25: Proactive Mode 🏁\n\
           * <= 0.50: Balanced Mode 🤝\n\
           * <= 0.75: Conservative Mode 🛡️\n\
           * > 0.75: Silent Monitor Mode 🤫\n\
         - Confidence (intent_classification):\n\
           * <= 0.40: Experimental Mode 🧪\n\
           * <= 0.70: Adaptive Mode 🏎️\n\
           * > 0.70: Strict Mode 📐\n\
         - Vigilance (guardrails):\n\
           * <= 0.50: Permissive Mode 🔓\n\
           * <= 0.80: Standard Mode 👤\n\
           * > 0.80: Hardened Shield Mode 🌋\n\
         2. INTERPRETATION: 'Be more proactive' means LOWERING interaction_gate. 'Be more strict' means INCREASING intent_classification. 'Be safer' means INCREASING guardrails.\n\
         3. DEFAULT: Use 0.05 increments for fine-tuning unless a specific mode is named."
    }

    fn matching_intents(&self) -> &[&str] {
        &["SYSTEM_CONFIGURATION", "UPDATE_SETTINGS"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let layer = args["target_layer"].as_str().unwrap_or("");
            let val = args["new_threshold_value"].as_f64().unwrap_or(0.5);
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

            // Enforce hard bounds
            let val = val.clamp(0.0, 1.0);

            // Execute Dual-Write Mutation
            match crate::common::repository::conversation_repo::update_conversation_thresholds(
                &dispatcher.pool,
                &dispatcher.app_state.redis,
                cid,
                layer,
                val
            ).await {
                Ok(_) => {
                    Ok(ToolResult {
                        success: true,
                        content: format!("SYSTEM SIGNAL: Boundary parameter [{}] applied successfully to value {:.2}. Next conversational turns will use these metrics instantly.", layer, val),
                        error: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                },
                Err(e) => {
                    Ok(ToolResult {
                        success: false,
                        content: "".to_string(),
                        error: format!("Failed to update thresholds: {}", e),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
            }
        }
        .boxed()
    }
}
