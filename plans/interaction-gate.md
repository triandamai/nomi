
 ### Task: Implement Isolated Ambient Group Chat Interaction Gate (New Service Only)


 We want to allow Nomi to intelligently evaluate and respond to ambient group chat messages without requiring an explicit `@mention`. We need to build a lightweight, vector-assisted **Interaction Gate Service** (`src/services/interaction_gate.rs`) to act as a pre-filtering node. It will determine if Nomi should respond *before* invoking our main 15k classification and orchestration pipelines.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT modify, alter, or overwrite any existing code.** This includes your active `IntentClassifier`, your active EMQX channel loops, your WhatsApp data model parsing files, and your Orchestrator.
 * **DO NOT handle tool mapping, text generation, or intent extraction here.** This service has exactly one isolated job: evaluate an inbound message context and return a boolean (`true`/`false`) flag.
 * **CREATE NEW FILES ONLY** to ensure a 100% safe refactoring boundaries.


 ---
 ### Step 1: Create the Isolated Interaction Gate Service


 Create a brand-new service file at `src/services/interaction_gate.rs`.
 Define an `InteractionGateService` struct that holds references to your database connection pool and your embedding engine service:
 ```rust
 pub struct InteractionGateService {
     pool: sqlx::Pool<sqlx::Postgres,
     // Reference to your embedding client wrapper goes here
 }
 
 ```


 Implement the primary asynchronous evaluation method:
 ```rust
 pub async fn should_respond_to_group_message(
     &self,
     message_body: &str,
     is_reply_to_nomi: bool,
 ) - anyhow::Result<bool
 
 ```


 ---


 ### Step 2: Implement the 3-Tier Evaluation Pass


 Inside `should_respond_to_group_message`, implement the following sequential evaluation gates:
 1. **Tier 1: Mechanical Fast-Pass (0 Token Cost)**
 * Convert `message_body` to lowercase.
 * If `is_reply_to_nomi == true` **OR** if the lowercase string contains the keyword `"nomi"`, short-circuit immediately, bypass all AI/database calls, and return `Ok(true)`.


 2. **Tier 2: Semantic Interaction Vector Query**
 * If the mechanical check fails, generate a semantic text embedding vector for the `message_body`.
 * Execute a vector similarity query against your Postgres `knowledge` table using these strict filters:
 * Filter explicitly where `type = 'interaction_triggers'`. (These represent expert context rules seeded in the database, e.g., *"Saat user menanyakan rekomendasi kuliner"* or *"Ketika grup membahas bug/error production"*).
 * Order by your vector distance metric and fetch the single closest match (`LIMIT 1`).




 3. **Tier 3: The Confidence Threshold Gate**
 * Extract the highest similarity score from the database query result.
 * If the result set is completely empty **OR** if the highest similarity score falls below a strict guard threshold (e.g., **`< 0.60`**), return `Ok(false)` immediately. This drops the message packet silently, saving your token limits.
 * If the score is **`= 0.60`**, return `Ok(true)`, signaling that Nomi has relevant context to chime into the conversation naturally.




 ---


 ### Step 3: Workspace Tree Registration


 * Register `src/services/interaction_gate.rs` inside your module tree (`src/services/mod.rs`).
 * Verify that the entire workspace compiles cleanly without triggering any dead-code warnings, asynchronous thread pool blocks, or trait boundary lifecycle compilation errors.


