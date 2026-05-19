
> ### Task: Implement Isolated Ambient Soul Service (Redis TTL) & Token Usage Telemetry Table Layout
>
>
> We need to build an asynchronous **Ambient Soul & Initiative Service** (`src/services/ambient_soul.rs`). This service will run on a background worker thread to passively extract long-term user memories from text logs and execute high-relevance proactive interaction loops. To ensure zero memory bloat on our server, we will use our existing Redis client to track per-conversation cooldown states using native Redis TTL expiration. Additionally, we need to create a dedicated ledger table infrastructure to store a historical timeline of all system token transactions for analytics, auditing, and system dashboard integration.
> #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES
>
>
> * **DO NOT modify, alter, or overwrite any existing code** running your active message-routing loops, EMQX channels, or your core database authentication models.
> * **DO NOT block the main message-routing loops.** All database writes for history auditing and ambient evaluations must execute as non-blocking asynchronous background worker tasks.
> * **CREATE NEW FILES / ISOLATED MIGRATIONS ONLY** to maintain 100% safe refactoring boundaries.
>
>
> ---
>
>
> ### Step 1: Create the Isolated Ambient Soul Service Structure
>
>
> Create a brand-new file at `src/services/ambient_soul.rs`. Define the structures to hold references to your Postgres pool and Redis connection clients, tracking metrics explicitly via dedicated types:
> ```rust
> use uuid::Uuid;
> use chrono::{DateTime, Utc};
> 
> #[derive(Debug, Clone, Default)]
> pub struct TokenMetrics {
>     pub input_tokens: u32,
>     pub output_tokens: u32,
>     pub total_tokens: u32,
> }
> 
> #[derive(Debug, Clone)]
> pub struct InitiativeResult {
>     pub response_text: Option<String>,
>     pub tokens: TokenMetrics,
> }
> 
> pub struct AmbientSoulService {
>     pool: sqlx::Pool<sqlx::Postgres>,
>     // Inject your project's active Redis client connection wrapper here
>     redis_client: crate::common::redis::RedisClient, 
>     // Include references to your embedding and gemini engines here
> }
> 
> ```
>
>
> #### Implement two asynchronous processing nodes within the service:
>
>
> 1. **Node A: Passive Memory Ingestion (`process_ambient_memory`)**
> ```rust
> pub async fn process_ambient_memory(
>     &self, 
>     user_id: Uuid, 
>     conversation_id: Uuid, 
>     conversation_log: &str
> ) -> anyhow::Result<TokenMetrics>
> 
> ```
>
>
> * **Logic:** Scan the input `conversation_log` using a low-temperature Gemini call to extract atomic long-term user facts. Intercept the response, pull the `usage_metadata` directly out of the Gemini API results stream, and map it into the `TokenMetrics` field.
> * If new facts are extracted, generate their semantic vectors and perform an **UPSERT injection** into the database table matching `type = 'knowledge'`, ensuring Nomi builds organic, persistent memory nodes over time. Return the processed `TokenMetrics`.
>
>
> 2. **Node B: Proactive Initiative Checker with Redis TTL (`evaluate_initiative`)**
> ```rust
> pub async fn evaluate_initiative(
>     &self, 
>     conversation_id: Uuid, 
>     current_message: &str, 
>     interaction_score: f64
> ) -> anyhow::Result<InitiativeResult>
> 
> ```
>
>
> * **Rule 1 (Per-Conversation Redis Cooldown Guard):** Generate an isolated Redis key matching `format!("nomi:cooldown:{}", conversation_id)`. If the key `EXISTS` in Redis, short-circuit immediately returning `InitiativeResult` with `response_text = None` and all token counts set to `0`.
> * **Rule 2 (Relevance Guard):** Enforce your strict threshold guards: skip if the filter's `interaction_score` is **`< 0.75`**.
> * **Rule 3 (Probability Roll):** Execute a fast random rolling check (e.g., a strict 20% success rate hurdle). If the roll fails, return `InitiativeResult` with `response_text = None` and token counts set to `0`.
> * **Inference & State Lock:** If all guards pass, run your context vector match, prompt Gemini to compile a highly natural comment, and extract the `usage_metadata` directly out of the response payload.
> * BEFORE returning, execute a Redis **`SET <key> "true" EX 900`** command to atomically create the cooldown key with an explicit **15-minute expiration time (900 seconds)**. Finally, return the full `InitiativeResult`.
>
>
>
>
> ---
>
>
> ### Step 2: Database Schema Refactoring (Token Usage History Ledger)
>
>
> Draft a clean SQL DDL structure or database migration module file to introduce our centralized telemetry ledger table:
> #### Table Definition: `token_usage_history`
>
>
> Define the migration using these explicit columns, data types, and nullability constraints:
> * `id`: Primary Key (UUID or Serial BigInt autoincrement)
> * `conversation_id`: UUID, **Nullable** (To track actions outside active chat sessions)
> * `message_id`: UUID, **Nullable** (To handle background worker or scheduler triggers)
> * `user_id`: UUID, **Nullable** (To handle system-level or unauthenticated execution spaces)
> * `type`: VARCHAR/TEXT, **Not Null** (Restricted logically to: `'knowledge'`, `'message'`, `'ambient_soul'`, `'schedular'`, `'reminder'`)
> * `role`: VARCHAR/TEXT, **Not Null** (Restricted logically to: `'assistant'`, `'system'`, `'user'`)
> * `input_tokens`: INT / BIGINT, **Not Null**
> * `output_tokens`: INT / BIGINT, **Not Null**
> * `total_tokens`: INT / BIGINT, **Not Null**
> * `created_at`: TIMESTAMPTZ, **Not Null**, Defaults to current system time (`NOW()`)
>
>
> #### Asynchronous Logging Integration Rules:
>
>
> * Write a helper database service method `log_token_transaction` that accepts this table's core parameters.
> * Ensure that whenever the system executes an update modifying the cumulative tokens field inside your primary `conversations` table, it **simultaneously executes a parallel, non-blocking asynchronous `INSERT**` statement writing an identical event copy into this new `token_usage_history` ledger.
>
>
> ---
>
>
> ### Step 3: Module Tree & Verification
>
>
> * Register the new module additions within your system module paths cleanly (`src/services/mod.rs`).
> * Execute a full workspace verification build sweep to ensure everything compiles flawlessly without triggering thread-safety issues, lifecycle errors, or database trait map constraints.
