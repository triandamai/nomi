use sqlx::{Pool, Postgres};
use tracing::{info, error};
use crate::rag::get_embedding;

pub struct InteractionGateService {
    pub pool: Pool<Postgres>,
    pub gemini_api_key: String,
}

impl InteractionGateService {
    pub fn new(pool: Pool<Postgres>, gemini_api_key: String) -> Self {
        Self {
            pool,
            gemini_api_key,
        }
    }

    /// Evaluates whether Nomi should respond to a message in a group chat context.
    /// Implements a 3-tier evaluation pass:
    /// 1. Mechanical Fast-Pass (Keyword/Reply detection)
    /// 2. Semantic Interaction Vector Query
    /// 3. Confidence Threshold Gate
    pub async fn should_respond_to_group_message(
        &self,
        message_body: &str,
        is_reply_to_nomi: bool,
    ) -> anyhow::Result<bool> {
        // Tier 1: Mechanical Fast-Pass (0 Token Cost)
        let body_lower = message_body.to_lowercase();
        if is_reply_to_nomi || body_lower.contains("nomi") {
            info!("Interaction Gate: Fast-pass triggered (is_reply={} or contains 'nomi')", is_reply_to_nomi);
            return Ok(true);
        }

        // Tier 2: Semantic Interaction Vector Query
        // If mechanical check fails, generate embedding for the message body
        let embedding_res = match get_embedding(&self.gemini_api_key, message_body).await {
            Ok(res) => res,
            Err(e) => {
                error!("Interaction Gate: Failed to generate embedding: {}", e);
                return Err(anyhow::anyhow!("Embedding error: {}", e));
            }
        };

        let vector = embedding_res.embedding.values;
        
        // Execute vector similarity query against knowledge_base
        // Filtering for type = 'interaction_triggers'
        let match_result = sqlx::query!(
            r#"
            SELECT 
                (1.0 - (embedding <=> $1::vector))::float8 as "score!"
            FROM knowledge_base
            WHERE metadata->>'type' = 'interaction_triggers'
            ORDER BY embedding <=> $1::vector
            LIMIT 1
            "#,
            vector as Vec<f32>
        )
        .fetch_optional(&self.pool)
        .await?;

        // Tier 3: The Confidence Threshold Gate
        match match_result {
            Some(row) => {
                let score = row.score;
                if score >= 0.60 {
                    info!("Interaction Gate: High confidence match found (score={:.4}), responding.", score);
                    Ok(true)
                } else {
                    info!("Interaction Gate: Match below threshold (score={:.4}), ignoring.", score);
                    Ok(false)
                }
            }
            None => {
                info!("Interaction Gate: No interaction triggers found in knowledge base.");
                Ok(false)
            }
        }
    }
}
