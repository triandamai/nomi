
 ### Task: Implement Isolated Multimodal Media Intent Interpreter Service


 We need to build a high-efficiency **Media Intent Interpreter Service** (`src/services/media_interpreter.rs`) to pre-process incoming messages containing S3 media URLs. Since our downstream vector classifiers are text-only, this service will intercept S3 media links, invoke Gemini’s multimodal capabilities to analyze the visual/audio content, and hydrate the text history payload with a rich description before it hits the `InteractionGate` or `IntentClassifier`.
 #### ⚠️ CRITICAL AGENT RULES: ZERO-TOUCH SAFETY BOUNDARIES


 * **DO NOT modify, alter, or overwrite any existing code.** This includes your active S3 upload logic, `IntentClassifier`, `InteractionGateService`, or `GuardrailService`.
 * **CREATE NEW FILES ONLY** to preserve 100% safe refactoring boundaries.


 ---


 ### Step 1: Create the Isolated Media Interpreter Service


 Create a brand-new file at `src/services/media_interpreter.rs`. Define the service struct to accept your Postgres connection pool, your S3 storage configuration parameters, and your active `Gemini` client framework:
 ```rust
 use uuid::Uuid;
 
 pub struct MediaInterpreterService {
     pool: sqlx::Pool<sqlx::Postgres,
     // Reference to your embedding and gemini engines go here
 }
 
 ```


 ---


 ### Step 2: Implement the Asynchronous Multimodal Hydration Node


 Implement the primary context transformation method within the service:
 ```rust
 pub async fn hydrate_media_context_string(
     &self,
     dispatcher: &crate::common::tools::ToolDispatcher,
     incoming_text_with_links: &str,
     mime_type: &str
 ) - anyhow::Result<(String, crate::services::intent_classifier::TokenMetrics)
 
 ```


 #### Implementation Flow inside the method:


 1. **URL Detection & Parsing:** Scan the `incoming_text_with_links` string to isolate the S3 asset path links and strip away any standalone captions.
 2. **Gemini Multimodal Inference:** Download or stream the attachment bytes from your S3 bucket. Pass the media buffer along to `dispatcher.gemini` using a specialized, fast system instruction prompt:
 * *For Images:* *"Analyze this image block completely. Extract all OCR text strings, transactional amounts, code exceptions, visible items, or environmental scenery. Summarize the content succinctly inside bracket markers."*
 * *For Audio Notes:* *"Transcribe the exact vocal spoken wording inside this clip cleanly into text."*


 3. **Text Context Synthesizer:** Take the generated description chunk and format it smoothly back into the text string alongside the user's captions. Example output structure: `"[Media Context Description: <Gemini Result] <User Text Caption"`.
 4. **Token Usage Logging Ledger:** Capture the exact `usage_metadata` counts from the Gemini API network completion stream.
 * Execute a parallel, non-blocking asynchronous database write inserting an identical telemetry log record straight into your `token_usage_history` table, marking the `type` column explicitly as `'knowledge'`.


 5. **Return Value:** Return a tuple containing the newly hydrated, text-rich contextual payload string along with the compiled transaction `TokenMetrics`.


 ---


 ### Step 3: Module Tree Integration


 * Register `src/services/media_interpreter.rs` inside your module tree (`src/services/mod.rs`).
 * Verify that the entire workspace builds cleanly without triggering compilation warnings or asynchronous thread blocking errors.
