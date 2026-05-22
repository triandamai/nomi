

# Master Architectural Blueprint: Context-Aware Agentic Tool Discovery Loop

## 🚨 Critical Agent Constraints & Safety Guardrails

* **DRY PRINCIPLE (Don't Repeat Yourself):** Reuse the existing `IntentClassifierService` and `build_system_prompt` closure variables already available inside the orchestrator scope. Do not rewrite parsing layers or duplicate data structs.
* **ZERO LIFETIME BREAKS:** Keep all futures pinned inside standard `BoxFuture` layouts when interacting across asynchronous multi-threaded boundaries.
* **NO DESTRUCTIVE REWRITES:** Leave the rest of the existing plugin maps, database connection parameters, and media buffers completely untouched.

---

## 🛠️ Step 1: Implement the Static System Meta-Plugin

Create a new file exactly at `gateway-rust/src/common/tools/plugins/discover_tools.rs` to act as Nomi's self-contained search tool.

```rust
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

```

---

## ⚙️ Step 2: Update ToolDispatcher to Always Force-Inject Discovery

Modify `gateway-rust/src/common/tools/mod.rs` (or where your `ToolDispatcher` is configured):

1. **Register the Plugin:** Inside `ToolDispatcher::new()`, register your plugin smoothly:
```rust
plugins.insert("discover_tools", std::sync::Arc::new(crate::common::tools::plugins::discover_tools::DiscoverToolsPlugin));

```


2. **Force Injection:** Modify `generate_tool_for_prompt` to push the schema unconditionally, skipping its normal intent evaluation loop:
```rust
pub async fn generate_tool_for_prompt(&self, intents: &[String]) -> Tool {
    let mut tools = Vec::new();

    // 🌟 ALWAYS FORCE INJECT DISCOVER_TOOLS NATIVELY
    if let Some(discover_plugin) = self.plugins.get("discover_tools") {
        let mut schema = discover_plugin.schema();
        Self::sanitize_schema_for_gemini(&mut schema);
        if let Ok(func_decl) = serde_json::from_value::<gemini_rust::FunctionDeclaration>(schema) {
            tools.push(func_decl);
        }
    }

    // 1. Static Plugable Tools (Skip discover_tools here to prevent duplication)
    for (name, plugin) in &self.plugins {
        if *name == "discover_tools" { continue; }
        // ... (keep your existing matching_intents loop logic completely unchanged) ...

```



---

## 🔄 Step 3: Integrate Loop Interception in V2Orchestrator

Open `gateway-rust/src/feature/message_processor/v2_orchestrator.rs`. Inside the `process_v2_message_with_intent` async state function, locate where `execute_tools(...).await;` returns.

Directly **below** the `execute_tools` execution line, insert this clean interception block to parse intents and re-hydrate the loop environment:

```rust
let tool_results = execute_tools(
    &dispatcher,
    current_calls.clone(),
    incoming_ctx.clone(),
    workspace_ctx.clone(),
).await;

// --- 🚨 START DISCOVERY INTERCEPTION HOOK 🚨 ---
let mut discovered_intents = Vec::new();
for (name, result) in &tool_results {
    if name == "discover_tools" && result.success {
        if let Some(start_idx) = result.content.find("[INTENT_INJECTION:") {
            if let Some(end_idx) = result.content[start_idx..].find(']') {
                let intents_raw = &result.content[start_idx + 18 .. start_idx + end_idx];
                for split_intent in intents_raw.split(',') {
                    let trimmed = split_intent.trim().to_string();
                    if !trimmed.is_empty() {
                        discovered_intents.push(trimmed);
                    }
                }
            }
        }
    }
}

if !discovered_intents.is_empty() {
    info!("🔄 Orchestrator successfully captured missing intents: {:?}", discovered_intents);
    for found_intent in discovered_intents {
        if !intents.contains(&found_intent) {
            intents.push(found_intent);
        }
    }
    // Automatically re-load the system prompt definitions with the new domain rules included!
    system_prompt = build_system_prompt(&intents);
}
// --- 🚨 END DISCOVERY INTERCEPTION HOOK 🚨 ---

```

---

### 🚀 Step 4: Run a Verification Build Pass

* Verify that the entire workspace builds cleanly without a single broken signature or lifetime mismatch error.

---