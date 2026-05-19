### Task: Implement Graph-Compatible Intent Classification Service with Token Tracking


We need to build a high-accuracy, token-optimized **Intent Classification Service** (`src/services/intent_classifier.rs`) for Nomi. This service must be structurally compatible with our RAG graph pipeline and explicitly track/log LLM token usage metrics for every classification operation.
The system will operate in a two-step hybrid layout:
1. **Boot-Time Sync:** Extract intent hooks from our registered plugins and cache them into our Postgres `knowledge` vector table.
2. **Runtime Classification (Graph Node):** Coarse-filter the database using vector embeddings of the user message + history, let a lightweight Gemini invocation act as the fine-grained judge, track token usage, and return a graph-compatible payload.


#### ⚠️ CRITICAL AGENT RULES: STRICT PRESERVATION BOUNDARIES


* **DO NOT modify, delete, or overwrite** any existing user table authentication workflows, database configuration migrations, or active `MqttManager` transport loops.
* **DO NOT handle tool execution or orchestration here.** The orchestrator or downstream graph nodes will handle execution based on the classification results.
* **CREATE NEW FILES ONLY** to encapsulate this classification module safely.

 
---


### Step 1: Data Structures & Boot-Time Ingestion


Create a brand-new file at `src/services/intent_classifier.rs`.
Define a structured output payload to return both the classification array and its associated token data so it can be consumed by our RAG graph state:
 ```rust
 #[derive(Debug, Clone)]
 pub struct ClassificationResult {
     pub intents: Vec<String,
     pub input_tokens: u32,
     pub output_tokens: u32,
     pub total_tokens: u32,
 }
 
 ```


Implement a boot-time synchronization function:
 ```rust
 pub async fn sync_plugin_intents_to_knowledge(dispatcher: &crate::common::tools::ToolDispatcher) - anyhow::Result<()
 
 ```


* **Logic:** Loop through all registered plugin traits inside `dispatcher.plugins` (`HashMap<&'static str, Arc<dyn NomiToolPlugin`).
* For each plugin, extract its string array from `plugin.matching_intents()`.
* For each intent string, generate a semantic vector embedding.
* Perform an **UPSERT SQL statement** into our `knowledge` table, setting `type = 'intent_classification'`. Store the raw intent token string inside the row's `metadata` JSON field (e.g., `{"intent": "SCHEDULE_TASK"}`). Ensure it avoids duplicate insertions if run multiple times.

 ---


### Step 2: Runtime Classification Graph Pipeline


Implement the main classification method designed to run inside our graph node execution flow:
 ```rust
 pub async fn classify_user_intent(
     &self,
     dispatcher: &crate::common::tools::ToolDispatcher,
     user_message: &str,
     chat_history_summary: &str
 ) - anyhow::Result<ClassificationResult
 
 ```

#### Implementation Flow inside the method:


1. **Context Vector Creation:** Combine `user_message` and `chat_history_summary` into a single semantic contextual payload string. Pass this combined string to the embedding engine to generate a search vector.
2. **Vector DB Query (Coarse Filtering):** Query the `knowledge` table:
* Filter explicitly where `type = 'intent_classification'`.
* Calculate distance/similarity math against the query vector.
* Fetch the top nearest matching records (`LIMIT 5`).

3. **The Similarity Threshold Guard Gate:** Evaluate the similarity score of the top database results.
* If the vector result array is empty, **OR** if the highest matching score falls below a confidence threshold (e.g., **`< 0.40`**), short-circuit immediately. Return a clean `ClassificationResult` containing `vec!["CHITCHAT".to_string()]` with all token usage counts set to `0` (since no LLM call was made).

4. **Gemini Finalization & Token Tracking:** If the score passes the gate, extract the candidate intent tokens from the database row metadata. Construct a lightweight system prompt for `dispatcher.gemini` containing the `chat_history_summary`, the `user_message`, and the candidate intents list.
* Execute the Gemini completion call.
* **Extract Usage Metadata:** Extract the `usage_metadata` (prompt_token_count, candidates_token_count, total_token_count) directly from the Gemini API response payload.
* Map the response text into a structured `Vec<String` and populate the fields of the final `ClassificationResult` struct.

### Step 3: Workspace Verification


* Ensure `src/services/intent_classifier.rs` is registered inside your module tree.
* Verify that the workspace compiles perfectly across all target modules without triggering any trait boundary conflicts, memory-safety warnings, or asynchronous thread block violations.