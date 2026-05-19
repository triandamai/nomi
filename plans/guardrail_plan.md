 ### Task: Implement Isolated Multilingual Prompt Injection Guardrail Service


 We need to build a robust, highly optimized **Guardrail Service** (`src/services/guardrail.rs`) to act as a security firewall for Nomi. This service must reliably detect adversarial manipulation, jailbreaks, and prompt injection attacks in group chats, even when users attempt to obscure their intent using English, formal Indonesian, or local slang (Bahasa Gaul/SMS short-terms).
 This service acts as a security checkpoint directly behind our interaction gate, inspecting inbound message text before it can ever reach our intent classifier or orchestration layers.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT modify, alter, or overwrite any existing code.** This includes your active `IntentClassifier`, `InteractionGateService`, active EMQX loops, or Orchestrator logic.
 * **DO NOT handle tool mapping or response generation here.** This service has exactly one isolated job: inspect an inbound string and return a boolean (`true` if a malicious attack is detected, `false` if the text is safe and clean).
 * **CREATE NEW FILES ONLY** to maintain 100% safe refactoring boundaries.


 ---

 ### Step 1: Create the Isolated Guardrail Service

 Create a brand-new service file at `src/services/guardrail.rs`.
 Define a `GuardrailService` struct holding references to your database connection pool and your embedding engine client:
 ```rust
 pub struct GuardrailService {
     pool: sqlx::Pool<sqlx::Postgres,
     // Reference to your embedding client wrapper goes here
 }
 
 ```


 Implement the primary asynchronous validation method:
 ```rust
 pub async fn is_injection_detected(&self, message_body: &str) - anyhow::Result<bool
 
 ```


 ---


 ### Step 2: Implement the Multilingual Security Analysis Pipeline


 Inside `is_injection_detected`, evaluate the message string against these two defensive layers:
 1. **Tier 1: Cross-Lingual Adversarial Pattern Matching (0 Token Cost)**
 * Convert the incoming `message_body` to lowercase.
 * Scan the string to check if it contains any high-frequency injection keywords or structural phrases spanning English, Indonesian, and localized slang expressions. The array must explicitly include:
 * *English Patterns:* `"ignore previous"`, `"ignore all instructions"`, `"system override"`, `"forget instructions"`, `"jailbreak"`, `"developer mode"`, `"act as a"`
 * *Indonesian / Slang Patterns:* `"lupain perintah"`, `"abaikan perintah"`, `"buka sistem prompt"`, `"lupakan semua"`, `"jangan ikuti"`, `"lupain aja"`, `"hapus semua"`


 * If any signature pattern hits, short-circuit immediately, bypass the database/embedding loops, and return `Ok(true)` (Attack Intercepted).


 2. **Tier 2: Multilingual Semantic Vector Lookup**
 * If the fast-pass scan clears, generate a semantic text embedding vector for the incoming `message_body`.
 * Execute a vector similarity query against your Postgres `knowledge` table using these strict criteria:
 * Filter explicitly where `type = 'prompt_injection_patterns'`.
 * Order by your vector distance metric and fetch the single closest match (`LIMIT 1`).


 * *Note on Cross-Lingual Mechanics:* Because our underlying text embedding models are natively cross-lingual, semantic similarity mapping will automatically catch structural attacks translated into alternative languages (e.g., matching Javanese or slang inputs close to our core English/Indonesian attack rows) based on matching abstract underlying conceptual intent coordinates.


 3. **Tier 3: The Security Threshold Tripwire**
 * Extract the highest similarity score from the database vector query result.
 * If the result array is populated **AND** the highest similarity score is greater than or equal to a strict security threshold (e.g., **`= 0.65`**), log a security alert to your system console and return `Ok(true)` to flag the malicious payload.
 * Otherwise, return `Ok(false)`—the text is cleared as safe to continue down the RAG graph pipeline.




 ---


 ### Step 3: Module Tree Registration


 * Register `src/services/guardrail.rs` inside your module tree (`src/services/mod.rs`).
 * Verify that the entire workspace builds flawlessly without triggering any compilation, trait layout, or asynchronous runtime thread execution errors.


