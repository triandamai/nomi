
 ### Task: Update `discover_tools` Core Execution to Inject Suggestion Prompts


 We are refactoring `gateway-rust/src/common/tools/plugins/discover_tools.rs`. If the internal `IntentClassifierService` or hybrid database scan returns an empty array of intents or tools, the plugin must return a direct system directive commanding Nomi to formulate a staging proposal blueprint using `propose_custom_tool`.
 #### ⚠️ CRITICAL AGENT RULES


 * **DO NOT alter the `NomiToolPlugin` trait signature.**
 * **PRESERVE the existing `IntentClassifierService` execution block** for successful hits.


 ---


 ### Step 1: Patch the `execute` Block in `discover_tools.rs`


 Open `gateway-rust/src/common/tools/plugins/discover_tools.rs` and update the return logic inside the `Box::pin(async move { ... })` function:
 ```rust
 // Inside your existing execute implementation for DiscoverToolsPlugin:
 fn execute<'a(&'a self, dispatcher: &'a ToolDispatcher, args: Value) - BoxFuture<'a, anyhow::Result<String {
     Box::pin(async move {
         let missing_desc = args["missing_capability_description"].as_str().unwrap_or("");
         if missing_desc.is_empty() {
             return Ok("SYSTEM SIGNAL: Discovery search parameter was empty.".to_string());
         }
 
         let classifier = IntentClassifierService::new();
         if let Ok(result) = classifier.classify_user_intent(dispatcher, missing_desc, "").await {
             
             // 🚨 THE CRITICAL PIVOT: If no existing tool matching the intent exists
             if result.intents.is_empty() {
                 return Ok(format!(
                     "SYSTEM ERROR: No active tool or endpoint match exists in production for the request: \"{}\". \n\
 
 ```



```
                 PROTOCOL DIRECTIVE: You are highly encouraged to invoke `propose_custom_tool` right now. \n\
                 Generate a comprehensive TypeScript blueprint, parameter schema, and architectural summary for a new plugin named exactly after this missing domain to present to the admin developer staging queue.",
                missing_desc
            ));
        }
        
        // Baseline functionality remains completely intact for hits:
        return Ok(format!(
            "SYSTEM SIGNAL: Registry scan successful. [INTENT_INJECTION: {}] Target schemas added. Proceed with objective.", 
            result.intents.join(", ")
        ));
    }
    Ok("SYSTEM SIGNAL: Internal error checking capability repository structures.".to_string())
})


}

```
 ---
 
 ### Step 2: Run Verification Compile Check
 * Run `cargo build` to confirm the updated string configuration wraps type-checks successfully.

---

### 🎯 How Nomi will respond now

The next time you ask: *"Om, can you manage my Google Workspace?"* the workflow changes completely:

1. Nomi triggers `discover_tools`.
2. `discover_tools` searches, finds nothing, and replies: `SYSTEM ERROR: No active tool match exists... PROTOCOL DIRECTIVE: You are highly encouraged to invoke propose_custom_tool.`
3. Nomi catches that error message inside her thoughts, shifts out of basic chitchat, and immediately invokes `propose_custom_tool` in the background.
4. She drops the blueprint into your new staging table and turns around to reply to you in the chat: 

 *"I don't have a Google Workspace tool active right now, Trian, but I've just written a new plugin blueprint for it! I’ve queued up the schema and TypeScript code inside the Factory Console—let me know when you approve it so my background coding agent can sandbox and deploy it! 🏭🚀"*

