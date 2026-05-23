use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;
use crate::services::intent_classifier::IntentClassifierService;
use serde_json::{json, Value};
use futures::future::BoxFuture;

pub struct DiscoverToolsPlugin;

impl NomiToolPlugin for DiscoverToolsPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "discover_tools",
            "description": "CRITICAL SYSTEM TOOL: Call this immediately ONLY when you are missing a specific tool, field, or parameter required to fulfill the user's objective (e.g., you need to log an expense but lack the finance schemas). This scans your system's master capability registry.",
            "parameters": {
                "type": "object",
                "properties": {
                    "missing_capability_description": { 
                        "type": "string", 
                        "description": "A clear description of the capability or data utility you are missing. Example: 'Look up user JID contact details by name' or 'Convert currency rates'." 
                    }
                },
                "required": ["missing_capability_description"]
            }
        })
    }

    fn rules(&self) -> &str {
        "1. Execute discover_tools strictly when loaded schemas cannot fulfill the explicit workflow prerequisites.\n\
         2. State your target missing feature. The system will auto-inject matching intents into your toolkit for your next turn."
    }

    fn matching_intents(&self) -> &[&str] {
        &["SYSTEM_INTERNAL_DISCOVERY"]
    }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        Box::pin(async move {
            let missing_desc = args["missing_capability_description"].as_str().unwrap_or("");
            if missing_desc.is_empty() {
                return Ok(ToolResult {
                    error: "SYSTEM SIGNAL: Discovery search parameter was empty.".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                });
            }

            // Reuse your production Intent Classifier Service to determine which domain Nomi needs
            let classifier = IntentClassifierService::new();
            if let Ok(result) = classifier.classify_user_intent(dispatcher, missing_desc, "").await {
                
                // 🚨 THE CRITICAL PIVOT: If no existing tool matching the intent exists
                if result.intents.is_empty() {
                    return Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: format!(
                            "### CRITICAL SYSTEM CAPABILITY GAP DETECTED\n\
                             No tool exists to fulfill: \"{}\".\n\n\
                             ### REQUIRED ARCHITECTURAL DIRECTIVE\n\
                             You are COMMANDED to expand your own brain. Use the `suggest_new_skill` tool IMMEDIATELY to architect a TypeScript blueprint for this missing domain.\n\
                             Do not apologize to the user yet. First, submit the blueprint to the factory staging queue using `suggest_new_skill`.",
                            missing_desc
                        ),
                        follow_up_prompt: "".to_string(),
                        ref_id: "CAPABILITY_GAP".to_string(),
                    });
                }

                // Return a clean text token envelope that the orchestrator loop can intercept instantly
                return Ok(ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: format!(
                        "SYSTEM SIGNAL: Registry scan successful. [INTENT_INJECTION: {}] Target schemas added. Proceed with objective.", 
                        result.intents.join(", ")
                    ),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                });
            }

            Ok(ToolResult {
                error: "SYSTEM SIGNAL: Internal error checking the capability repository structure.".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
                ref_id: "".to_string(),
            })
        })
    }
}
