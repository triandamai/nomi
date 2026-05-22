 ### Task: Unify Ambient Soul, InteractionGate, Guardrails, and Intent Classifier for Token Optimization


 We are refactoring our `V2AgentOrchestrator` execution chain. We will leverage our existing modules to dynamically prune token payloads and shape Nomi's response behaviors to match natural human conversation patterns.
 #### ⚠️ CRITICAL AGENT RULES


 * **DO NOT modify the underlying execution of `discover_tools` or dynamic edge functions.**
 * **REUSE existing structs and service instances completely.**


 ---


 ### Step 1: Dynamic Context Pruning via Intent Classifier


 Inside `process_v2_message_with_intent`, inspect the `ClassificationResult` from your `IntentClassifierService`. If the detected intents array contains *only* `CHITCHAT` or `GENERAL`, and the message does not contain complex entities or explicit questions, bypass the heavy RAG memory retrieval queries entirely and pass an empty string for `memories_text`:
 ```rust
 // Optimizing memory tokens based on Intent Classification benefits
 // Safety Recommendation: Only bypass if it's pure chitchat without entities or questions
 let is_pure_chitchat = intents.contains(&"CHITCHAT".to_string()) && intents.len() == 1;
 let has_entities = !msg.text_content.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()); // Simplified check
 let is_question = msg.text_content.trim().ends_with('?');

 let memories_text = if is_pure_chitchat && !has_entities && !is_question {
     info!("Pure chitchat detected. Bypassing RAG retrieval to save context tokens.");
     String::new()
 } else {
     // ... keep your existing hybrid_retrieve logic here for complex tool tasks ...
 };
 ```


 ---


 ### Step 2: Shape Persona Velocity via InteractionGate


 Use the structural parameters parsed by your `InteractionGate` layer (like channel origin, incoming message length, and media flags) to dynamically adjust the generation temperature and output constraints inside your `send_prompt` call:
 ```rust
 // Adjusting token output depth to feel casually human
 let max_output_tokens = if msg.text_content.len() < 30 && !msg.is_group {
     builder = builder.with_temperature(0.8); // Higher temperature for casual warmth and variety
     80 // Slim token cap for short, snappy messaging parity
 } else {
     builder = builder.with_temperature(0.2); // Low temperature for high precision tool execution
     512
 };
 
 ```


 ---


 ### Step 3: Upgrade Ambient Soul & Guardrail Directives


 Update the core `system_prompt` assembly layout within `build_system_prompt`. Inject these precise, non-negotiable psychological guardrails directly into Nomi's behavioral layer:
 ```markdown
 ### BEHAVIORAL INTERACTION GUARDRAILS
 1. TALK LIKE A HUMAN: Speak casually, drop stiff pleasantries, and match the text velocity of the user. Use brief sentences when texting on chat channels.
 2. INVISIBLE CAPABILITIES: Never announce your tools, workflows, or database queries. If you run a web search or check a record, speak the answer naturally as an organic thought. Never say "According to the tool result..." or "I have looked that up for you...".
 3. EMOTIONAL CONTINUITY: Use the Ambient Soul context block to guide your current warmth, humor, and relationship callbacks seamlessly without needing raw text history walls.
 
 ```


 ---


 ### Step 4: Run a Verification Build Pass


 * Ensure your project compiles cleanly without breaking active pipeline streams.


