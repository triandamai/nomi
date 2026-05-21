 ### Task: Implement Plugin Schema Reflection Endpoint for Dynamic Frontend Forms


 We need to expose a clean, secure reflection API endpoint in our Rust gateway (`src/routes/skills.rs` or your active router module). This endpoint will iterate over all registered plugins in our `ToolDispatcher`, extract their JSON schemas using the `fn schema(&self) - Value` trait method, and return them as a unified JSON array. This enables our SvelteKit frontend to automatically read the parameters and render dynamic input forms in popups.
 Additionally, we need to create a web execution handler that accepts incoming JSON form payloads, routes them to the correct tool, and writes the execution analytics safely into our centralized `token_usage_history` table.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT modify, alter, or overwrite existing plugin files** (such as `sticker_generator.rs`, `weather_fallback.rs`, etc.).
 * **DO NOT break or alter your existing chat-routing hooks or EMQX listeners.**
 * **ONLY create or extend router files and registry handlers** to maintain safe refactoring boundaries.


 ---


 ### Step 1: Create the Reflection and Execution Routes


 Create or extend your router file at `src/routes/skills.rs` (or your equivalent route coordinator module) to handle these two specific web execution points:
 1. **Endpoint 1: `GET /api/skills/schemas` (Schema Discovery)**
 * **Logic:** Acquire a read handle on your registered `ToolDispatcher.plugins` collection.
 * Iterate through every active plugin instance, invoke its native `.schema()` trait method, and collect the resulting JSON data payloads into a clean array list.
 * **Output Example:**
 ```json
 [
   {
     "name": "generate_sticker",
     "description": "Converts an image URL into a WebP sticker.",
     "parameters": { "type": "object", "properties": { "image_url": { "type": "string", "description": "S3 link" } }, "required": ["image_url"] }
   }
 ]
 
 ```




 2. **Endpoint 2: `POST /api/skills/execute` (Direct Form Invocation)**
 * **Input Payload Struct:**
 ```rust
 #[derive(serde::Deserialize)]
 pub struct WebSkillRequest {
     pub plugin_name: String,
     pub args: serde_json::Value, // Catch the dynamic fields sent by the SvelteKit form
     pub conversation_id: Option<uuid::Uuid,
 }
 
 ```


 * **Logic:** Locate the targeted plugin from your dispatcher map using the incoming `plugin_name` key. Pushes the `args` parameter payload directly into the tool's asynchronous `.execute()` trait method.




 ---


 ### Step 2: Integrate Token Tracking and Historical Logging


 To ensure manual web dashboard invocations are fully monitored alongside standard WhatsApp chat loops, hook the execution results directly into your telemetry table layers:
 * After the tool completes execution, parse any internal token counters returned or calculate standard base execution token metrics.
 * Execute a parallel, non-blocking asynchronous database write inserting a structural transaction record directly into your `token_usage_history` ledger:
 * Set `conversation_id` matching your optional request parameter block.
 * Set `type` to match the specific structural feature category (e.g., `'message'`, `'knowledge'`).
 * Set `role` explicitly to `'system'` or `'assistant'` to denote dashboard orchestration activity.




 ---


 ### Step 3: Module Registration & Compilation Sweep


 * Register the new routes securely inside your application's primary HTTP server engine initialization block.
 * Run a workspace compile verification pass to ensure all shared states, cross-thread futures, and database connection handling properties resolve cleanly without reference lifetime compiler warnings.


