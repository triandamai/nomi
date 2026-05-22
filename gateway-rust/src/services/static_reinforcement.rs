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

    let raw_json = response.text().trim().replace("```json", "").replace("```", "").trim().to_string();
    if raw_json.is_empty() { return Ok(()); }
    
    let parsed: serde_json::Value = serde_json::from_str(&raw_json)?;
    let enriched_desc = parsed["enriched_description"].as_str().unwrap_or(&base_description).to_string();
    let new_rule = parsed["new_rule"].as_str().unwrap_or("").to_string();
    let phrase = parsed["phrase"].as_str().unwrap_or("").to_string();

    // 🚨 PROMPT BLOAT GUARDRAIL: Upsert enforces a strict FIFO limit (max 5 rules, 10 phrases) via database arrays
    sqlx::query(
        "INSERT INTO static_plugin_reinforcements (plugin_slug, enriched_description, additional_rules, learned_phrases, updated_at) \
         VALUES ($1, $2, ARRAY[$3::text], ARRAY[$4::text], NOW()) \
         ON CONFLICT (plugin_slug) DO UPDATE SET \
            enriched_description = EXCLUDED.enriched_description, \
            additional_rules = id_stable_array_append_fifo(static_plugin_reinforcements.additional_rules, $3::text, 5), \
            learned_phrases = id_stable_array_append_fifo(static_plugin_reinforcements.learned_phrases, $4::text, 10), \
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
