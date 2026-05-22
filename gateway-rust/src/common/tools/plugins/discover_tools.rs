use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
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

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, anyhow::Result<String>> {
        Box::pin(async move {
            let missing_desc = args["missing_capability_description"].as_str().unwrap_or("");
            if missing_desc.is_empty() {
                return Ok("SYSTEM SIGNAL: Discovery search parameter was empty.".to_string());
            }

            // Reuse your production Intent Classifier Service to determine which domain Nomi needs
            let classifier = IntentClassifierService::new();
            if let Ok(result) = classifier.classify_user_intent(dispatcher, missing_desc, "").await {
                if result.intents.is_empty() {
                    return Ok("SYSTEM SIGNAL: No matching capability domains found in the registry.".to_string());
                }

                // Return a clean text token envelope that the orchestrator loop can intercept instantly
                return Ok(format!(
                    "SYSTEM SIGNAL: Registry scan successful. [INTENT_INJECTION: {}] Target schemas added. Proceed with objective.", 
                    result.intents.join(", ")
                ));
            }

            Ok("SYSTEM SIGNAL: Internal error checking the capability repository structure.".to_string())
        })
    }
}
