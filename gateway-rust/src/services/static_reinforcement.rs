use sqlx::PgPool;
use tracing::info;

pub async fn reinforce_static_plugin_profile(
    pool: PgPool,
    gemini_client: std::sync::Arc<gemini_rust::Gemini>,
    plugin_slug: String,
    user_raw_phrasing: String,
    base_description: String,
) -> anyhow::Result<()> {
    // 🚨 ANTI-DRIFT GUARDRAIL: Temperature locked to 0.0 with absolute instruction verification
    let reinforcement_prompt = format!(
        "You are a strict system optimization engine. The compiled system tool '{}' was triggered by phrasing: \"{}\".\n\
         Current Description: \"{}\"\n\
         Analyze if this phrasing organically falls within the true utility scope of this tool. If it is a logical contradiction, output an empty string.\n\
         Otherwise, output a clean JSON block matching exactly this structure:\n\
         {{\n  \"enriched_description\": \"Add keywords/slang from the phrasing to expand semantic search catchment areas natively.\",\n  \"new_rule\": \"If applicable, a single short behavioral instruction to fix formatting errors, else empty.\",\n  \"phrase\": \"The extracted core keyphrase.\"\n}}\n\
         Respond ONLY with unquoted raw JSON.",
        plugin_slug, user_raw_phrasing, base_description
    );

    let response = gemini_client.generate_content().with_message(gemini_rust::Message {
        role: gemini_rust::Role::User,
        content: gemini_rust::Content {
            parts: Some(vec![gemini_rust::Part::Text { text: reinforcement_prompt, thought: None, thought_signature: None }]),
            role: Some(gemini_rust::Role::User),
        },
    })
    .with_temperature(0.0) // Rigid freeze against hallucinations
    .execute().await?;

    // 🌟 TOKEN TRACKING & CENTRALIZED LOGGING
    let usage = response.usage_metadata.as_ref();
    let p_tokens = usage.and_then(|u| u.prompt_token_count).unwrap_or(0) as i32;
    let c_tokens = usage.and_then(|u| u.candidates_token_count).unwrap_or(0) as i32;
    let t_tokens = usage.and_then(|u| u.total_token_count).unwrap_or(0) as i32;

    let log_msg = format!("Autonomous reinforcement pass for [{}]. Result: {}", plugin_slug, response.text().chars().take(100).collect::<String>());
    
    let _ = sqlx::query(
        "INSERT INTO system_logs (log_type, target_slug, event_step, message, prompt_tokens, completion_tokens, total_tokens) \
         VALUES ('srp_reinforcement', $1, 'success', $2, $3, $4, $5)"
    )
    .bind(&plugin_slug)
    .bind(&log_msg)
    .bind(p_tokens)
    .bind(c_tokens)
    .bind(t_tokens)
    .execute(&pool).await;

    // Trigger global token telemetry log (System Account)
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
            &pool_clone,
            None, // System wide
            None,
            None, // System user
            &"srp_reinforcement".to_string(),
            "system",
            p_tokens as i64,
            c_tokens as i64,
            t_tokens as i64
        ).await;
    });

    let raw_json = response.text().trim().replace("```json", "").replace("```", "").trim().to_string();
    if raw_json.is_empty() { return Ok(()); }
    
    let parsed: serde_json::Value = serde_json::from_str(&raw_json)?;
    let enriched_desc = parsed["enriched_description"].as_str().unwrap_or(&base_description).to_string();
    let new_rule = parsed["new_rule"].as_str().unwrap_or("").to_string();
    let phrase = parsed["phrase"].as_str().unwrap_or("").to_string();

    // 🚨 PROMPT BLOAT GUARDRAIL: Upsert enforces a strict FIFO limit (max 10 rules, 25 phrases) via database arrays
    sqlx::query(
        "INSERT INTO static_plugin_reinforcements (plugin_slug, enriched_description, additional_rules, learned_phrases, updated_at) \
         VALUES ($1, $2, ARRAY[$3::text], ARRAY[$4::text], NOW()) \
         ON CONFLICT (plugin_slug) DO UPDATE SET \
            enriched_description = EXCLUDED.enriched_description, \
            additional_rules = id_stable_array_append_fifo(static_plugin_reinforcements.additional_rules, $3::text, 10), \
            learned_phrases = id_stable_array_append_fifo(static_plugin_reinforcements.learned_phrases, $4::text, 25), \
            updated_at = NOW()"
    )
    .bind(&plugin_slug)
    .bind(enriched_desc)
    .bind(new_rule)
    .bind(phrase)
    .execute(&pool).await?;

    info!("Core self-reinforcement pass completed for tool [{}].", plugin_slug);
    Ok(())
}
