use sqlx::{Pool, Postgres};
use uuid::Uuid;
use tracing::{info, error};
use serde_json::json;
use crate::common::redis::RedisClient;
use gemini_rust::Gemini;
use std::sync::Arc;
use crate::rag::get_embedding;
use redis::AsyncCommands;

#[derive(Debug, Clone, Default)]
pub struct TokenMetrics {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct InitiativeResult {
    pub response_text: Option<String>,
    pub tokens: TokenMetrics,
}

pub struct AmbientSoulService {
    pool: sqlx::Pool<sqlx::Postgres>,
    redis_client: RedisClient,
    gemini: Arc<Gemini>,
    gemini_api_key: String,
}

impl AmbientSoulService {
    pub fn new(
        pool: Pool<Postgres>,
        redis_client: RedisClient,
        gemini: Arc<Gemini>,
        gemini_api_key: String,
    ) -> Self {
        Self {
            pool,
            redis_client,
            gemini,
            gemini_api_key,
        }
    }

    /// Logs a token transaction to the telemetry ledger table
    pub async fn log_token_transaction(
        pool: &Pool<Postgres>,
        conversation_id: Option<Uuid>,
        message_id: Option<Uuid>,
        user_id: Option<Uuid>,
        log_type: &str,
        role: &str,
        input_tokens: i64,
        output_tokens: i64,
        total_tokens: i64,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO token_usage_history 
            (conversation_id, message_id, user_id, type, role, input_tokens, output_tokens, total_tokens)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(conversation_id)
        .bind(message_id)
        .bind(user_id)
        .bind(log_type)
        .bind(role)
        .bind(input_tokens)
        .bind(output_tokens)
        .bind(total_tokens)
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub async fn process_ambient_memory(
        &self,
        _user_id: Uuid,
        conversation_id: Uuid,
        conversation_log: &str,
    ) -> anyhow::Result<TokenMetrics> {
        info!("AmbientSoul: Processing ambient memory for conversation {}", conversation_id);
        
        let system_prompt = "You are a passive memory extractor. Analyze the following conversation log and extract any concrete, long-term facts, preferences, or important events about the user. Return them as a concise bulleted list. If no meaningful long-term facts are present, return exactly 'NO_FACTS'.";
        
        let res = self.gemini.generate_content()
            .with_system_prompt(system_prompt)
            .with_user_message(conversation_log)
            // Low temperature for factual extraction
            .with_temperature(0.1) 
            .execute()
            .await?;

        let mut metrics = TokenMetrics::default();
        if let Some(usage) = &res.usage_metadata {
            metrics.input_tokens = usage.prompt_token_count.unwrap_or(0) as u32;
            metrics.output_tokens = usage.candidates_token_count.unwrap_or(0) as u32;
            metrics.total_tokens = usage.total_token_count.unwrap_or(0) as u32;
        }

        let extracted_text = res.text().trim().to_string();

        if extracted_text != "NO_FACTS" && !extracted_text.is_empty() {
            info!("AmbientSoul: Extracted new facts. Generating embedding and saving to knowledge base.");
            // Generate semantic vector
            if let Ok(embedding_res) = get_embedding(&self.gemini_api_key, &extracted_text).await {
                let metadata = json!({
                    "type": "ambient_memory",
                    "conversation_id": conversation_id.to_string(),
                    "source": "ambient_soul_worker"
                });

                // UPSERT injection via existing helper
                let _ = crate::rag::save_to_knowledge_base(
                    &self.pool,
                    &extracted_text,
                    embedding_res.embedding.values,
                    Some(metadata),
                    Some(conversation_id),
                    metrics.input_tokens as i32,
                    metrics.output_tokens as i32,
                    metrics.total_tokens as i32
                ).await?;

                // Update Cache tokens
                let total_updated: i32 = sqlx::query_scalar!(
                    "SELECT cumulative_tokens FROM conversations WHERE id = $1",
                    conversation_id
                )
                .fetch_one(&self.pool)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);

                crate::common::repository::conversation_repo::update_cached_tokens(
                    &self.redis_client,
                    conversation_id,
                    total_updated
                ).await;
            }
        } else {
            info!("AmbientSoul: No new facts extracted.");
        }

        Ok(metrics)
    }

    pub async fn evaluate_initiative(
        &self,
        conversation_id: Uuid,
        current_message: &str,
        interaction_score: f64,
    ) -> anyhow::Result<InitiativeResult> {
        // Rule 1: Per-Conversation Redis Cooldown Guard
        let cooldown_key = format!("nomi:cooldown:{}", conversation_id);
        
        let mut conn = match self.redis_client.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("AmbientSoul: Redis error getting connection: {}", e);
                return Ok(InitiativeResult {
                    response_text: None,
                    tokens: TokenMetrics::default(),
                });
            }
        };

        // Use EXISTS directly
        let is_cooling_down: bool = conn.exists(&cooldown_key).await.unwrap_or_else(|e| {
            error!("AmbientSoul: Redis error checking cooldown: {}", e);
            false
        });

        if is_cooling_down {
            info!("AmbientSoul: Conversation {} is in cooldown. Skipping initiative.", conversation_id);
            return Ok(InitiativeResult {
                response_text: None,
                tokens: TokenMetrics::default(),
            });
        }

        // Rule 2: Relevance Guard
        // Lowered threshold to 0.60 to align with Interaction Gate
        if interaction_score < 0.60 {
            info!("AmbientSoul: Interaction score ({}) below threshold (0.60). Skipping.", interaction_score);
            return Ok(InitiativeResult {
                response_text: None,
                tokens: TokenMetrics::default(),
            });
        }

        // Rule 3: Dynamic Probability Roll
        // Check for momentum (Nomi's recent participation)
        let recent_participation = sqlx::query!(
            "SELECT count(*) as count FROM (
                SELECT role FROM messages 
                WHERE conversation_id = $1 
                ORDER BY created_at DESC 
                LIMIT 5
            ) as recent WHERE role = 'assistant'",
            conversation_id
        )
        .fetch_one(&self.pool)
        .await?;

        let has_momentum = recent_participation.count.unwrap_or(0) > 0;
        
        // Base probability is 30%, if we have momentum, it bumps to 50%
        let threshold = if has_momentum { 0.50 } else { 0.30 };

        let roll: f64 = rand::random();
        if roll > threshold {
            info!("AmbientSoul: Probability roll failed ({:.2} > {:.2}, momentum={}). Skipping.", roll, threshold, has_momentum);
            return Ok(InitiativeResult {
                response_text: None,
                tokens: TokenMetrics::default(),
            });
        }

        // Inference & State Lock
        info!("AmbientSoul: All guards passed (score={:.2}, momentum={}). Generating proactive response for {}", interaction_score, has_momentum, conversation_id);
        
        let system_prompt = "You are Nomi, a proactive and highly observant AI partner. You have chosen to speak up in a group conversation based on the context. Provide a very short, natural, and witty comment (1-2 sentences maximum). Do not use placeholders. Just say the comment.";
        
        let res = self.gemini.generate_content()
            .with_system_prompt(system_prompt)
            .with_user_message(current_message)
            .execute()
            .await?;

        let mut metrics = TokenMetrics::default();
        if let Some(usage) = &res.usage_metadata {
            metrics.input_tokens = usage.prompt_token_count.unwrap_or(0) as u32;
            metrics.output_tokens = usage.candidates_token_count.unwrap_or(0) as u32;
            metrics.total_tokens = usage.total_token_count.unwrap_or(0) as u32;
        }

        let response_text = res.text().trim().to_string();

        // State Lock: Set Redis EX 900 (15 minutes)
        if let Err(e) = self.redis_client.set_ex(&cooldown_key, "true", 900).await {
            error!("AmbientSoul: Failed to set cooldown lock: {}", e);
        }

        Ok(InitiativeResult {
            response_text: Some(response_text),
            tokens: metrics,
        })
    }
}
