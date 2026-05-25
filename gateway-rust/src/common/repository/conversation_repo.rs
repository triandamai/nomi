use crate::common::redis::RedisClient;
use crate::feature::Conversation;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use tracing::{info, error};
use serde_json::Value;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationCache {
    pub id: Uuid,
    pub title: Option<String>,
    pub user_id: Option<Uuid>,
    pub soul_content: Option<String>,
    pub bootstrap_content: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub max_token_usage: i32,
    pub cumulative_tokens: i32,
    pub conversation_type: String,
    pub metadata: Option<Value>,
}

impl From<ConversationCache> for Conversation {
    fn from(cache: ConversationCache) -> Self {
        Self {
            id: cache.id,
            session_id: cache.user_id,
            title: cache.title,
            soul_content: cache.soul_content,
            bootstrap_content: cache.bootstrap_content,
            created_at: Some(cache.created_at),
            updated_at: Some(cache.updated_at),
        }
    }
}

pub async fn get_conversation_info(
    pool: &Pool<Postgres>,
    redis: &RedisClient,
    conversation_id: Uuid,
) -> anyhow::Result<ConversationCache> {
    let cache_key = format!("nomi:conversation:{}", conversation_id);

    // 1. Try Redis Cache
    match redis.get(&cache_key).await {
        Ok(Some(cached)) => {
            match serde_json::from_str::<ConversationCache>(&cached) {
                Ok(info) => {
                    info!("Cache HIT for key: {}", cache_key);
                    return Ok(info);
                }
                Err(e) => {
                    error!("Cache HIT but DESERIALIZATION FAILED for key {}: {}", cache_key, e);
                    // Continue to fallback
                }
            }
        }
        Ok(None) => {
            info!("Cache MISS (key not found) for key: {}", cache_key);
        }
        Err(e) => {
            error!("Cache ERROR for key {}: {}", cache_key, e);
        }
    }

    // 2. Fallback to Postgres
    info!("Fetching conversation {} from Postgres.", conversation_id);
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            title,
            user_id,
            soul_content,
            bootstrap_content,
            created_at,
            updated_at,
            max_token_usage,
            cumulative_tokens,
            metadata,
            conversation_type
        FROM conversations WHERE id = $1
        "#,
        conversation_id
    )
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let info = ConversationCache {
                id: r.id,
                title: r.title,
                user_id: r.user_id,
                soul_content: r.soul_content,
                bootstrap_content: r.bootstrap_content,
                created_at: r.created_at.unwrap_or_else(Utc::now),
                updated_at: r.updated_at.unwrap_or_else(Utc::now),
                max_token_usage: r.max_token_usage.unwrap_or(700000),
                cumulative_tokens: r.cumulative_tokens.unwrap_or(0),
                conversation_type: r.conversation_type,
                metadata: r.metadata,
            };

            // 3. Save to Redis (Expire in 1 hour)
            match serde_json::to_string(&info) {
                Ok(serialized) => {
                    info!("Saving conversation {} to Redis cache...", conversation_id);
                    match redis.set_ex(&cache_key, &serialized, 3600).await {
                        Ok(_) => info!("Successfully cached conversation {} in Redis.", conversation_id),
                        Err(e) => error!("Failed to save conversation {} to Redis: {}", conversation_id, e),
                    }
                }
                Err(e) => error!("Failed to serialize conversation {} for caching: {}", conversation_id, e),
            }

            Ok(info)
        }
        None => Err(anyhow::anyhow!("Conversation not found")),
    }
}

pub async fn update_cached_tokens(redis: &RedisClient, conversation_id: Uuid, new_tokens: i32) {
    let cache_key = format!("nomi:conversation:{}", conversation_id);
    
    if let Ok(Some(cached)) = redis.get(&cache_key).await {
        if let Ok(mut info) = serde_json::from_str::<ConversationCache>(&cached) {
            info.cumulative_tokens = new_tokens;
            if let Ok(serialized) = serde_json::to_string(&info) {
                let _ = redis.set_ex(&cache_key, &serialized, 3600).await;
            }
        }
    }
}

pub async fn invalidate_conversation_cache(redis: &RedisClient, conversation_id: Uuid) {
    let cache_key = format!("nomi:conversation:{}", conversation_id);
    let _ = redis.del(&cache_key).await;
    info!("Invalidated cache for conversation {}", conversation_id);
}
