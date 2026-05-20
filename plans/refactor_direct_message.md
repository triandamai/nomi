
 ### Task: Refactor Direct Messaging Tool for Native ID Resolution and Context Retention


 We need to refactor our outbound messaging tool plugin (e.g., `src/plugins/send_direct_message.rs`) and its corresponding trait rules to offload network ID validation from Gemini into our native Rust execution block. The tool must dynamically handle destination variables whether they are Postgres UUIDs, WhatsApp phone JIDs (`@s.whatsapp.net`), or Linked Identities (`@lid`), while ensuring outbound messages are logged to the database for downstream RAG context preservation.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT alter or overwrite any external gateway files** running your active EMQX listeners.
 * **MODIFY ONLY** your messaging tool plugin file and its module integration declarations.
 * Ensure all database queries and network publish calls execute inside non-blocking asynchronous routines.


 ---


 ### Step 1: Update the Tool Schema & LLM Rules Contract


 Inside your plugin file, update the `schema()` and `rules()` trait methods so Gemini knows it doesn't need to look for a raw JID—any valid ID format returned by a user lookup tool is safe to pass:
 ```rust
 fn schema(&self) - Value {
     json!({
         "name": "send_direct_message",
         "description": "Fires an outbound message payload to a specified target. Accepts a database UUID string, a standard phone JID, or a protocol LID string as the target parameter.",
         "parameters": {
             "type": "object",
             "properties": {
                 "target": {
                     "type": "string",
                     "description": "The target identifier. Can be a raw database UUID string, an @s.whatsapp.net string, or an @lid string."
                 },
                 "message_body": {
                     "type": "string",
                     "description": "The exact textual content of the outbound message to transmit."
                 }
             },
             "required": ["target", "message_body"]
         }
     })
 }
 
 fn rules(&self) - &str {
     "OUTBOUND MESSAGING CORE RULES:\n\
     1. When instructed to send a message to a user, always call your user lookup/search tool first.\n\
     2. As soon as the search tool returns an identifier (whether it is a database UUID string, JID, or LID), pass that identifier directly into the 'target' field of this tool.\n\
     3. Do not halt execution or request additional format conversions if the target identifier looks like a UUID or a non-phone string; the native tool wrapper has full responsibility for resolving routing addresses."
 }
 
 ```


 ---


 ### Step 2: Implement Dynamic ID Resolution inside `execute`


 Inside the `execute` block of your plugin, handle the incoming `target` string dynamically before compiling your EMQX network message payload:
 ```rust
 // Inside execute future block:
 let target = args["target"].as_str().ok_or_else(|| anyhow::anyhow!("Missing target block"))?;
 let message_body = args["message_body"].as_str().ok_or_else(|| anyhow::anyhow!("Missing message_body block"))?;
 
 let resolved_jid = if let Ok(parsed_uuid) = uuid::Uuid::parse_str(target) {
     // 🅰️ TARGET IS A UUID: Run a clean database query against your users/channels table 
     // to extract the active, routable phone JID associated with this primary key ID.
     database::resolve_jid_from_user_uuid(&self.pool, parsed_uuid).await?
         .ok_or_else(|| anyhow::anyhow!("No active channel JID mapped to UUID: {}", parsed_uuid))?
 } else if target.ends_with("@lid") {
     // 🅱️ TARGET IS A PROTOCOL LID: Run your lookup to map the LID string back
     // to a standard, deliverable @s.whatsapp.net phone address.
     database::lookup_phone_by_lid(&self.pool, target).await?
         .unwrap_or_else(|| target.to_string()) // Fallback gracefully if database misses
 } else {
     // 🅲 TARGET IS ALREADY A ROUTABLE JID: Pass it through natively
     target.to_string()
 };
 
 ```


 ---


 ### Step 3: Implement Context Retention Logging (The Conversation Memory Anchor)


 To ensure Nomi maintains absolute context when the target user receives the message and decides to text back, your tool must record the outbound event into your relational timeline:
 * **Log to Message Database:** Right after successfully publishing your outbound text packet to your EMQX broker topic, execute an asynchronous database write inserting a new row into your `messages` table:
 * Set `conversation_id` matching the target chat context.
 * Set `sender_id` to Nomi's platform identity string.
 * Set `message_body` to the generated text content.
 * Set `role` or `type` explicitly to indicate it was an outbound assistant transaction.


 * **Why this is critical:** When the user replies to this message, your upstream `chat_history_summary` query will naturally load this row as the immediate preceding turn, giving Gemini perfect contextual awareness of what Nomi said previously so she can reply smoothly without repeating herself.


 ---


 ### Step 4: Verification Build


 * Run a full workspace compile pass to guarantee your tool registers, handles its interior database joins cleanly, and builds without reference lifetime errors.

