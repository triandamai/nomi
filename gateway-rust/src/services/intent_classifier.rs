use crate::common::tools::ToolDispatcher;
use crate::rag::get_embedding;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error};

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
    pub async fn sync_plugin_intents_to_knowledge(dispatcher: &ToolDispatcher) -> anyhow::Result<()> {
        info!("Starting boot-time intent synchronization...");
        let api_key = &dispatcher.gemini_api_key;
        
        for (name, plugin) in &dispatcher.plugins {
            let intents = plugin.matching_intents();
            for intent in intents {
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
                    info!("Syncing new intent from plugin {}: {}", name, intent);
                    let embedding_res = match get_embedding(api_key, intent).await {
                        Ok(res) => res,
                        Err(e) => {
                            error!("Failed to generate embedding for intent {}: {}", intent, e);
                            continue;
                        }
                    };
                    
                    let metadata = json!({
                        "type": "intent_classification",
                        "intent": intent,
                        "plugin": name
                    });

                    let _ = crate::rag::save_to_knowledge_base(
                        &dispatcher.pool,
                        intent,
                        embedding_res.embedding.values,
                        Some(metadata),
                        None, // Global intent, no conversation_id
                        0, 0, 0
                    ).await?;
                }
            }
        }
        info!("Intent synchronization complete.");
        Ok(())
    }

    pub async fn classify_user_intent(
        &self,
        dispatcher: &ToolDispatcher,
        user_message: &str,
        chat_history_summary: &str
    ) -> anyhow::Result<ClassificationResult> {
        // 1. Context Vector Creation
        let context_payload = format!("History: {}\nMessage: {}", chat_history_summary, user_message);
        let embedding_res = match get_embedding(&dispatcher.gemini_api_key, &context_payload).await {
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
        if candidates.is_empty() || candidates[0].score < 0.40 {
            info!("Intent classification: Below threshold ({:.4}), returning CHITCHAT", 
                candidates.get(0).map(|c| c.score).unwrap_or(0.0));
            return Ok(ClassificationResult {
                intents: vec!["CHITCHAT".to_string()],
                input_tokens: 0,
                output_tokens: 0,
                total_tokens: 0,
            });
        }

        let candidate_names: Vec<&str> = candidates.iter().map(|c| c.intent.as_str()).collect();
        
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

        let res = dispatcher.gemini.generate_content()
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
