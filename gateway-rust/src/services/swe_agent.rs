use std::collections::HashMap;
use sqlx::PgPool;
use serde_json::json;
use crate::AppState;
use tracing::info;

pub fn process_factory_build(pool: PgPool, state: AppState, slug: String) -> futures::future::BoxFuture<'static, anyhow::Result<()>> {
    use futures::FutureExt;
    async move {
        info!("[DAF]: Initiating autonomous build pass for tool handle: [{}]", slug);
        let suggestion_res = sqlx::query(
            "SELECT id,name, description, schema_json, how_it_works FROM plugin_creation_suggestions WHERE slug = $1 LIMIT 1"
        )
            .bind(&slug)
            .fetch_one(&pool)
            .await?;

        use sqlx::Row;
        let name: String = suggestion_res.get("name");
        let id: String = suggestion_res.get("id");
        let description: String = suggestion_res.get("description");
        let schema_json: serde_json::Value = suggestion_res.get("schema_json");
        let how_it_works: String = suggestion_res.get("how_it_works");

        // ⚡ DECOUPLED EVENT BUS EMISSION SYSTEM FOR EVOLUTION MATCHING
        let emit_factory_event = |step: &str, log_msg: &str, active_code: &str| {
            let current_slug = slug.clone();
            info!("[DAF-TELEMETRY] [{}]: {}", current_slug, log_msg);

            // 🌟 PERSISTENT CENTRALIZED LOGGING
            let pool_log = pool.clone();
            let msg_log = log_msg.to_string();
            let step_log = step.to_string();
            let slug_log = current_slug.clone();
            let id = id.clone();
            tokio::spawn(async move {
                let meta = serde_json::json!({
                    "ref_id": id
                });
                let _ = sqlx::query(
                    "INSERT INTO system_logs (log_type, target_slug, event_step, message,metadata) \
                     VALUES ('swe_build', $1, $2, $3,$4)"
                )
                    .bind(slug_log)
                    .bind(step_log)
                    .bind(msg_log)
                    .bind(meta)
                    .execute(&pool_log).await;
            });

            let payload = json!({ 
                "slug": current_slug, 
                "step": step, 
                "log": log_msg, 
                "code": active_code 
            });
            let state_clone = state.clone();
            tokio::spawn(async move {
                let event = crate::services::event_dispatcher::AppEvent::broadcast(
                    "evolution",
                    payload,
                );
                let _ = crate::services::event_dispatcher::dispatch(&state_clone, event).await;
            });
        };

        emit_factory_event("thinking", "[FACTORY]: SWE Agent awakened. Synchronizing with Skill Creation Protocol...", "");

        // 🌟 PROTOCOL SYNCHRONIZATION: Read the latest standards from SKILLS.md
        let protocol_paths = ["./docs/SKILLS.md", "../docs/SKILLS.md"];
        let mut protocol_doc = String::new();
        for path in protocol_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                protocol_doc = content;
                break;
            }
        }

        let swe_system_prompt = format!(
            "You are an elite automated backend architect specializing in serverless V8 and Bun runtime environments. \
            Your output must be 100% executable TypeScript code block statements without introductory filler or markdown chatter. \
            \n\n### ARCHITECTURAL PROTOCOL STANDARDS:\n{}\n\n\
            CRITICAL STRUCTURE RULE: You MUST export a default function matching exactly the entrypoint format defined in the protocol. \
            The `args` parameter maps directly to your input properties schema matrix variables. Ensure all data processing happens inside this function framework.",
            protocol_doc
        );

        let mut coding_prompt = format!(
            "Synthesize a complete, production-grade TypeScript edge plugin file. Specifications:\n\
             Name: {}\nDescription: {}\nParameters Expected Schema: {}\nFunctional Roadmap: {}\n\
             Respond ONLY with the raw typescript execution code inside standard backtick fences, starting with the default export run function layout.",
            name, description, schema_json, how_it_works
        );

        let mut attempt = 0;
        let max_retries = 3;

        while attempt < max_retries {
            attempt += 1;
            emit_factory_event("thinking", &format!("[SYNTHESIS RUN {}/{}]: Composing runtime script variables...", attempt, max_retries), "");

            let response = state.gemini.generate_content().with_message(gemini_rust::Message {
                role: gemini_rust::Role::User,
                content: gemini_rust::Content {
                    parts: Some(vec![gemini_rust::Part::Text { text: coding_prompt.clone(), thought: None, thought_signature: None }]),
                    role: Some(gemini_rust::Role::User),
                },
            })
                .with_system_prompt(swe_system_prompt.to_string())
                .with_temperature(0.0)
                .execute().await?;

            // 🌟 TOKEN TRACKING & PERSISTENT LOGGING
            let usage = response.usage_metadata.as_ref();
            let p_tokens = usage.and_then(|u| u.prompt_token_count).unwrap_or(0) as i32;
            let c_tokens = usage.and_then(|u| u.candidates_token_count).unwrap_or(0) as i32;
            let t_tokens = usage.and_then(|u| u.total_token_count).unwrap_or(0) as i32;

            let log_msg = format!("[SYNTHESIS PASS {}]: Generated {} tokens.", attempt, t_tokens);
            let pool_clone = pool.clone();
            let slug_clone = slug.clone();
            tokio::spawn(async move {
                let _ = sqlx::query(
                    "INSERT INTO system_logs (log_type, target_slug, event_step, message, prompt_tokens, completion_tokens, total_tokens) \
                     VALUES ('swe_build', $1, 'thinking', $2, $3, $4, $5)"
                )
                    .bind(slug_clone)
                    .bind(log_msg)
                    .bind(p_tokens)
                    .bind(c_tokens)
                    .bind(t_tokens)
                    .execute(&pool_clone).await;

                // Trigger global token telemetry log (System Account)
                let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
                    &pool_clone,
                    None, None, None,
                    &"swe_agent_build".to_string(),
                    "system",
                    p_tokens as i64,
                    c_tokens as i64,
                    t_tokens as i64,
                ).await;
            });

            let code = response.text().trim().replace("```typescript", "").replace("```", "").trim().to_string();

            // 🌟 AUTONOMOUS SANDBOX VALIDATION
            emit_factory_event("sandboxing", &format!("[SANDBOX]: Initiating runtime validation pass {}...", attempt), &code);

            // 1. Generate Sample Arguments based on Schema
            let sample_args_prompt = format!(
                "Generate a single JSON object containing realistic sample arguments that adhere strictly to this tool schema: {}. \
                 Return ONLY the raw JSON object, no markdown fences, no introductory text.",
                schema_json
            );

            let args_res = state.gemini.generate_content().with_message(gemini_rust::Message {
                role: gemini_rust::Role::User,
                content: gemini_rust::Content {
                    parts: Some(vec![gemini_rust::Part::Text { text: sample_args_prompt, thought: None, thought_signature: None }]),
                    role: Some(gemini_rust::Role::User),
                },
            })
                .with_temperature(0.0)
                .execute().await?;

            let sample_args_raw = args_res.text().trim().replace("```json", "").replace("```", "").trim().to_string();
            let sample_args: serde_json::Value = serde_json::from_str(&sample_args_raw).unwrap_or(serde_json::json!({}));

            // 2. Execute in Sandbox
            let executor = crate::common::tools::edge_runner::BunEdgeExecutor {
                slug: slug.clone(),
                script_code: code.clone(),
            };

            let bridge_token = "DAF_SWE_TEST_TOKEN";
            let api_base_url = "http://localhost:8000";

            let incoming = serde_json::json!({
                "is_group": false,
                "is_mentioned": true,
                "sender_id": "swe_agent",
                "conversation_id": uuid::Uuid::nil(),
                "text": "Autonomous Sandbox Test",
                "channel": "sandbox"
            });

            let workspace = serde_json::json!({
                "id": uuid::Uuid::nil(),
                "title": "SWE Factory Sandbox"
            });

            let env: HashMap<String, String> = HashMap::new();
            let test_res = executor.run(bridge_token, api_base_url, sample_args, incoming, workspace, env).await;

            let (test_pass_success, test_pass_output) = match test_res {
                Ok(res) => (true, format!("Execution Successful. Output: {}\nLogs: {}", res.result, res.logs)),
                Err(e) => (false, format!("Runtime Exception:\n{}", e))
            };

            if test_pass_success {
                // 3. 🌟 AUTONOMOUS EVALUATION PASS
                emit_factory_event("thinking", "[EVALUATION]: Analyzing runtime output alignment with functional roadmap...", "");
                let evaluation_prompt = format!(
                    "As a senior QA architect, evaluate the runtime outcome of the tool blueprint for [{}].\n\
                     Roadmap Objective: {}\n\
                     Sandbox Execution Result: {}\n\n\
                     Does this output logically fulfill the intended functionality? Answer with 'PASSED' or describe the logical failure in 1 sentence.",
                    slug, how_it_works, test_pass_output
                );

                let eval_res = state.gemini.generate_content().with_message(gemini_rust::Message {
                    role: gemini_rust::Role::User,
                    content: gemini_rust::Content {
                        parts: Some(vec![gemini_rust::Part::Text { text: evaluation_prompt, thought: None, thought_signature: None }]),
                        role: Some(gemini_rust::Role::User),
                    },
                }).with_temperature(0.0).execute().await?;

                let evaluation = eval_res.text().trim().to_uppercase();

                if evaluation.contains("PASSED") {
                    emit_factory_event("success", "[VALIDATION SUCCESS]: Code structures and logical output verified cleanly. Ready for production deployment.", &code);

                    let _ = sqlx::query(
                        "UPDATE plugin_creation_suggestions SET compiled_code = $1, status = 'ready', error_logs = NULL, updated_at = NOW() WHERE slug = $2"
                    )
                        .bind(&code)
                        .bind(&slug)
                        .execute(&pool)
                        .await?;

                    check_and_trigger_next_queued_plugin(pool.clone(), state.clone()).await;
                    return Ok(());
                } else {
                    emit_factory_event("healing", &format!("[LOGIC FAILURE]: Output did not align with roadmap. Discrepancy: {}", evaluation), &code);

                    // 🌟 PERSIST LOGIC ERROR FOR AUDIT
                    let error_msg = format!("[Logic Pass {} Failure]: {}", attempt, evaluation);
                    let _ = sqlx::query("UPDATE plugin_creation_suggestions SET error_logs = COALESCE(error_logs, '') || $1 || '\n', updated_at = NOW() WHERE slug = $2")
                        .bind(&error_msg).bind(&slug).execute(&pool).await;

                    coding_prompt = format!(
                        "Your previous code synthesized correctly but produced a logical discrepancy during evaluation.\n\
                         Roadmap: {}\n\
                         Evaluation Feedback: {}\n\n\
                         Correct the implementation to strictly follow the roadmap roadmap and re-output code.",
                        how_it_works, evaluation
                    );
                    continue;
                }
            }

            emit_factory_event("healing", &format!("[SANDBOX TRACE ERROR]: Runtime check failed. Initiating self-correction mechanics...\n\n{}", test_pass_output), &code);

            // 🌟 PERSIST RUNTIME ERROR FOR AUDIT
            let error_msg = format!("[Sandbox Pass {} Failure]: {}", attempt, test_pass_output);
            let _ = sqlx::query("UPDATE plugin_creation_suggestions SET error_logs = COALESCE(error_logs, '') || $1 || '\n', updated_at = NOW() WHERE slug = $2")
                .bind(&error_msg).bind(&slug).execute(&pool).await;

            // Log the failure to system_logs for audit
            let log_msg = format!("Sandbox Failure on pass {}: {}", attempt, test_pass_output);
            let pool_log = pool.clone();
            let slug_log = slug.clone();
            tokio::spawn(async move {
                let _ = sqlx::query("INSERT INTO system_logs (log_type, target_slug, event_step, message) VALUES ('swe_build', $1, 'healing', $2)")
                    .bind(slug_log).bind(log_msg).execute(&pool_log).await;
            });

            coding_prompt = format!(
                "Your previous code configuration attempt failed compilation or execution check rules.\n\
                 ---- SANDBOX COMPILER STACK EXCEPTION LOGS ----\n{}\n\n\
                 Review the error parameters carefully, preserve the required `export default function run(args)` signature, correct your bugs, and re-output code.",
                test_pass_output
            );
        }

        emit_factory_event("failed", "[ABORTED]: Failed execution validation gates within maximum retry thresholds.", "");
        let _ = sqlx::query(
            "UPDATE plugin_creation_suggestions SET status = 'failed', updated_at = NOW() WHERE slug = $1"
        )
            .bind(&slug)
            .execute(&pool)
            .await?;

        check_and_trigger_next_queued_plugin(pool, state).await;
        Ok(())
    }.boxed()
}

async fn check_and_trigger_next_queued_plugin(pool: PgPool, state: AppState) {
    let next_res = sqlx::query(
        "UPDATE plugin_creation_suggestions SET status = 'processing', updated_at = NOW() WHERE id = (SELECT id FROM plugin_creation_suggestions WHERE status = 'approved' ORDER BY updated_at ASC LIMIT 1) RETURNING slug"
    )
        .fetch_optional(&pool)
        .await;

    if let Ok(Some(row)) = next_res {
        use sqlx::Row;
        let next_slug: String = row.get("slug");
        tokio::spawn(async move {
            let _ = process_factory_build(pool, state, next_slug).await;
        });
    }
}
