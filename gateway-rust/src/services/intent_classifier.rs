use crate::common::tools::ToolDispatcher;
use crate::feature::edge_functions::CreateEdgeFunctionRequest;
use crate::rag::get_embedding;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub intents: Vec<String>,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

pub struct IntentClassifierService;

impl IntentClassifierService {
    pub fn new() -> Self {
        IntentClassifierService {}
    }
    pub async fn sync_plugin_intents_to_knowledge(
        dispatcher: &ToolDispatcher,
    ) -> anyhow::Result<()> {
        info!("Starting boot-time intent synchronization...");
        let api_key = &dispatcher.gemini_api_key;

        for (name, plugin) in &dispatcher.plugins {
            let intents = plugin.matching_intents();
            let schema = plugin.schema();
            let description = schema["description"].as_str().unwrap_or_default();

            for intent in intents {
                // Capability-based text for better semantic overlap
                let capability_text = format!("Intent: {}. Description: {}", intent, description);

                // Check if already exists to avoid duplicates
                let exists = sqlx::query!(
                    r#"
                    SELECT id FROM knowledge_base 
                    WHERE metadata->>'type' = 'intent_classification' 
                    AND metadata->>'intent' = $1
                    "#,
                    intent
                )
                .fetch_optional(&dispatcher.pool)
                .await?;

                if exists.is_none() {
                    info!("Syncing new capability for plugin {}: {}", name, intent);
                    let embedding_res = match get_embedding(api_key, &capability_text).await {
                        Ok(res) => res,
                        Err(e) => {
                            error!("Failed to generate embedding for intent {}: {}", intent, e);
                            continue;
                        }
                    };

                    let metadata = json!({
                        "type": "intent_classification",
                        "intent": intent,
                        "plugin": name,
                        "description": description
                    });

                    let _ = crate::rag::save_to_knowledge_base(
                        &dispatcher.pool,
                        &capability_text,
                        embedding_res.embedding.values,
                        Some(metadata),
                        None, // Global intent, no conversation_id
                        0,
                        0,
                        0,
                    )
                    .await?;
                }
            }
        }
        info!("Intent synchronization complete.");
        Ok(())
    }

    pub async fn sync_dynamic_plugin_intents_to_knowledge(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        gemini_api_key: &str,
        payload: &CreateEdgeFunctionRequest,
        edge_function_id: Uuid,
    ) -> anyhow::Result<()> {
        info!("Syncing capabilities for dynamic plugin: {}", payload.name);

        // 1. Clear existing links for this function to avoid orphans
        sqlx::query(
            "DELETE FROM knowledge_edge_function WHERE edge_function_id = $1"
        )
        .bind(edge_function_id)
        .execute(&mut **tx).await?;

        for intent in &payload.intents {
            let capability_text = format!("Intent: {}. Description: {}", intent, payload.description);

            // Check if a capability for this intent already exists in knowledge_base
            #[derive(sqlx::FromRow)]
            struct KnowledgeIdRow { id: Uuid }

            let row = sqlx::query_as::<_, KnowledgeIdRow>(
                r#"
                    SELECT id FROM knowledge_base
                    WHERE metadata->>'type' = 'intent_classification'
                    AND metadata->>'intent' = $1
                    "#
            )
            .bind(intent)
            .fetch_optional(&mut **tx)
            .await?;

            let embedding_res = match get_embedding(
                gemini_api_key,
                &capability_text,
            )
            .await
            {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to generate embedding for intent {}: {}", intent, e);
                    continue;
                }
            };

            let metadata = json!({
                "type": "intent_classification",
                "intent": intent,
                "plugin": payload.name,
                "description": payload.description,
                "slug": payload.slug
            });

            let knowledge_id = match row {
                Some(r) => {
                    // Update existing capability record
                    sqlx::query(
                        "UPDATE knowledge_base SET content = $1, embedding = $2, metadata = $3 WHERE id = $4"
                    )
                    .bind(&capability_text)
                    .bind(embedding_res.embedding.values as Vec<f32>)
                    .bind(&metadata)
                    .bind(r.id)
                    .execute(&mut **tx).await?;
                    r.id
                },
                None => {
                    // Insert new capability record
                    let k_id = Uuid::new_v4();
                    sqlx::query(
                        "INSERT INTO knowledge_base (id, content, embedding, metadata, prompt_tokens, answer_tokens, total_tokens) VALUES ($1, $2, $3, $4, 0, 0, 0)"
                    )
                    .bind(k_id)
                    .bind(&capability_text)
                    .bind(embedding_res.embedding.values as Vec<f32>)
                    .bind(&metadata)
                    .execute(&mut **tx).await?;
                    k_id
                }
            };

            // 2. Link the capability to the edge function
            sqlx::query(
                "INSERT INTO knowledge_edge_function (knowledge_id, edge_function_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
            )
            .bind(knowledge_id)
            .bind(edge_function_id)
            .execute(&mut **tx).await?;
        }

        Ok(())
    }

    pub async fn classify_user_intent(
        &self,
        dispatcher: &ToolDispatcher,
        user_message: &str,
        chat_history_summary: &str,
        thresholds: &serde_json::Value,
    ) -> anyhow::Result<ClassificationResult> {
        // 1. Context Vector Creation
        let context_payload = format!(
            "History: {}\nMessage: {}",
            chat_history_summary, user_message
        );
        let embedding_res = match get_embedding(&dispatcher.gemini_api_key, &context_payload).await
        {
            Ok(res) => res,
            Err(e) => return Err(anyhow::anyhow!("Embedding error: {}", e)),
        };

        // 2. Vector DB Query (Coarse Filtering)
        let vector = embedding_res.embedding.values;
        let candidates = sqlx::query!(
            r#"
            SELECT 
                metadata->>'intent' as "intent!",
                (1.0 - (embedding <=> $1::vector))::float8 as "score!"
            FROM knowledge_base
            WHERE metadata->>'type' = 'intent_classification'
            ORDER BY embedding <=> $1::vector
            LIMIT 5
            "#,
            vector as Vec<f32>
        )
        .fetch_all(&dispatcher.pool)
        .await?;

        // 3. Similarity Threshold Guard Gate
        // DEB: Read baseline from thresholds, fallback to 0.40
        let threshold = thresholds["intent_classification"].as_f64().unwrap_or(0.40);

        if candidates.is_empty() || candidates[0].score < threshold {
            info!(
                "Intent classification: Below threshold ({:.4}, target={:.2}), returning CHITCHAT",
                candidates.get(0).map(|c| c.score).unwrap_or(0.0),
                threshold
            );
            return Ok(ClassificationResult {
                intents: vec!["CHITCHAT".to_string()],
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            });
        }

        let candidate_names: Vec<&str> = candidates.iter().map(|c| c.intent.as_str()).collect();

        info!("Candidate Intent: {}", candidate_names.join(", "));
        // 4. Gemini Finalization & Token Tracking
        let system_prompt = format!(
            "You are an intent classifier for an AI assistant. \
            Analyze the user message and history to determine which of the candidate intents apply. \
            Return a comma-separated list of matching intents from the candidates list ONLY. \
            If none of the candidates match, return CHITCHAT. \n\n\
            Candidates: [{}]",
            candidate_names.join(", ")
        );

        let user_prompt = format!(
            "History: {}\nUser Message: {}",
            chat_history_summary, user_message
        );

        let res = dispatcher
            .gemini
            .generate_content()
            .with_system_prompt(system_prompt)
            .with_user_message(user_prompt)
            .execute()
            .await?;

        let usage = res.usage_metadata.as_ref();
        let input_tokens = usage.and_then(|u| u.prompt_token_count).unwrap_or(0) as u32;
        let output_tokens = usage.and_then(|u| u.candidates_token_count).unwrap_or(0) as u32;
        let total_tokens = usage.and_then(|u| u.total_token_count).unwrap_or(0) as u32;

        let response_text = res.text().to_uppercase();
        let intents: Vec<String> = response_text
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(ClassificationResult {
            intents,
            input_tokens,
            output_tokens,
            total_tokens,
        })
    }
}
