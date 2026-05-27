use crate::common::redis::RedisClient;
use crate::feature::Conversation;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;
use tracing::{info, error};
use serde_json::{json, Value};
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
    pub gateway_thresholds: Value,
}

impl From<ConversationCache> for Conversation {
    fn from(cache: ConversationCache) -> Self {
        Self {
            id: cache.id,
            session_id: cache.user_id,
            title: cache.title,
            soul_content: cache.soul_content,
            bootstrap_content: cache.bootstrap_content,
            gateway_thresholds: Some(cache.gateway_thresholds),
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

    // 2. Fallback to Postgres (Using runtime query to avoid migration sync issues during build)
    info!("Fetching conversation {} from Postgres.", conversation_id);
    let row = sqlx::query(
        "SELECT id, title, user_id, soul_content, bootstrap_content, created_at, updated_at, 
                max_token_usage, cumulative_tokens, metadata, conversation_type, gateway_thresholds 
         FROM conversations WHERE id = $1"
    )
    .bind(conversation_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some(r) => {
            let info = ConversationCache {
                id: r.get("id"),
                title: r.get("title"),
                user_id: r.get("user_id"),
                soul_content: r.get("soul_content"),
                bootstrap_content: r.get("bootstrap_content"),
                created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").unwrap_or_else(Utc::now),
                updated_at: r.get::<Option<DateTime<Utc>>, _>("updated_at").unwrap_or_else(Utc::now),
                max_token_usage: r.get::<Option<i32>, _>("max_token_usage").unwrap_or(700000),
                cumulative_tokens: r.get::<Option<i32>, _>("cumulative_tokens").unwrap_or(0),
                conversation_type: r.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string()),
                metadata: r.get("metadata"),
                gateway_thresholds: r.get::<Option<Value>, _>("gateway_thresholds").unwrap_or_else(|| json!({
                    "interaction_gate": 0.6,
                    "intent_classification": 0.4,
                    "guardrails": 0.65
                })),
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

pub async fn update_conversation_thresholds(
    pool: &Pool<Postgres>,
    redis: &RedisClient,
    conversation_id: Uuid,
    layer: &str,
    value: f64,
) -> anyhow::Result<Value> {
    // 1. Update Postgres (JSONB Patch) - Using robust COALESCE
    let row = sqlx::query(
        "UPDATE conversations 
         SET gateway_thresholds = COALESCE(gateway_thresholds, '{}'::jsonb) || jsonb_build_object($1::text, $2::float8), 
             updated_at = NOW() 
         WHERE id = $3 
         RETURNING gateway_thresholds"
    )
    .bind(layer)
    .bind(value)
    .bind(conversation_id)
    .fetch_one(pool)
    .await?;

    let updated_thresholds: Value = row.get("gateway_thresholds");

    // 2. Invalidate Redis Cache (ensure a clean miss loads up-to-date Postgres values next turn)
    invalidate_conversation_cache(redis, conversation_id).await;

    Ok(updated_thresholds)
}

pub async fn invalidate_conversation_cache(redis: &RedisClient, conversation_id: Uuid) {
    let cache_key = format!("nomi:conversation:{}", conversation_id);
    let _ = redis.del(&cache_key).await;
    info!("Invalidated cache for conversation {}", conversation_id);
}

pub async fn get_user_conversations(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    limit: Option<i64>,
) -> anyhow::Result<Vec<ConversationCache>> {
    let limit = limit.unwrap_or(50);
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.title, cm.user_id, c.soul_content, c.bootstrap_content, c.created_at, c.updated_at, 
               c.max_token_usage, c.cumulative_tokens, c.metadata, c.conversation_type, c.gateway_thresholds
        FROM conversations c
        INNER JOIN conversation_members cm ON c.id = cm.conversation_id
        WHERE cm.user_id = $1 AND (c.conversation_type IS NULL OR c.conversation_type != 'channel_subchat')
        ORDER BY c.updated_at DESC
        LIMIT $2
        "#
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut conversations = Vec::new();
    for r in rows {
        conversations.push(ConversationCache {
            id: r.get("id"),
            title: r.get("title"),
            user_id: r.get("user_id"),
            soul_content: r.get("soul_content"),
            bootstrap_content: r.get("bootstrap_content"),
            created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").unwrap_or_else(Utc::now),
            updated_at: r.get::<Option<DateTime<Utc>>, _>("updated_at").unwrap_or_else(Utc::now),
            max_token_usage: r.get::<Option<i32>, _>("max_token_usage").unwrap_or(700000),
            cumulative_tokens: r.get::<Option<i32>, _>("cumulative_tokens").unwrap_or(0),
            conversation_type: r.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string()),
            metadata: r.get("metadata"),
            gateway_thresholds: r.get::<Option<Value>, _>("gateway_thresholds").unwrap_or_else(|| json!({
                "interaction_gate": 0.6,
                "intent_classification": 0.4,
                "guardrails": 0.65
            })),
        });
    }

    Ok(conversations)
}

pub async fn get_sub_conversations(
    pool: &Pool<Postgres>,
    parent_id: Uuid,
) -> anyhow::Result<Vec<ConversationCache>> {
    let rows = sqlx::query(
        r#"
        SELECT id, title, user_id, soul_content, bootstrap_content, created_at, updated_at, 
               max_token_usage, cumulative_tokens, metadata, conversation_type, gateway_thresholds
        FROM conversations
        WHERE parent_id = $1
        ORDER BY updated_at DESC
        "#
    )
    .bind(parent_id)
    .fetch_all(pool)
    .await?;

    let mut conversations = Vec::new();
    for r in rows {
        conversations.push(ConversationCache {
            id: r.get("id"),
            title: r.get("title"),
            user_id: r.get("user_id"),
            soul_content: r.get("soul_content"),
            bootstrap_content: r.get("bootstrap_content"),
            created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").unwrap_or_else(Utc::now),
            updated_at: r.get::<Option<DateTime<Utc>>, _>("updated_at").unwrap_or_else(Utc::now),
            max_token_usage: r.get::<Option<i32>, _>("max_token_usage").unwrap_or(700000),
            cumulative_tokens: r.get::<Option<i32>, _>("cumulative_tokens").unwrap_or(0),
            conversation_type: r.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string()),
            metadata: r.get("metadata"),
            gateway_thresholds: r.get::<Option<Value>, _>("gateway_thresholds").unwrap_or_else(|| json!({
                "interaction_gate": 0.6,
                "intent_classification": 0.4,
                "guardrails": 0.65
            })),
        });
    }

    Ok(conversations)
}

pub async fn create_conversation(
    pool: &Pool<Postgres>,
    id: Uuid,
    title: String,
    soul_content: Option<String>,
    bootstrap_content: Option<String>,
    conversation_type: Option<String>,
    user_id: Uuid,
) -> anyhow::Result<ConversationCache> {
    let mut tx = pool.begin().await?;
    let conv_type = conversation_type.unwrap_or_else(|| "private".to_string());

    let r = sqlx::query(
        r#"
        INSERT INTO conversations (id, title, soul_content, bootstrap_content, cumulative_tokens, conversation_type) 
        VALUES ($1, $2, $3, $4, 0, $5) 
        RETURNING id, title, user_id, soul_content, bootstrap_content, created_at, updated_at, 
                  max_token_usage, cumulative_tokens, metadata, conversation_type, gateway_thresholds
        "#
    )
    .bind(id)
    .bind(&title)
    .bind(&soul_content)
    .bind(&bootstrap_content)
    .bind(conv_type)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2)"
    )
    .bind(id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let cache = ConversationCache {
        id: r.get("id"),
        title: r.get("title"),
        user_id: r.get("user_id"),
        soul_content: r.get("soul_content"),
        bootstrap_content: r.get("bootstrap_content"),
        created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").unwrap_or_else(Utc::now),
        updated_at: r.get::<Option<DateTime<Utc>>, _>("updated_at").unwrap_or_else(Utc::now),
        max_token_usage: r.get::<Option<i32>, _>("max_token_usage").unwrap_or(700000),
        cumulative_tokens: r.get::<Option<i32>, _>("cumulative_tokens").unwrap_or(0),
        conversation_type: r.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string()),
        metadata: r.get("metadata"),
        gateway_thresholds: r.get::<Option<Value>, _>("gateway_thresholds").unwrap_or_else(|| json!({
            "interaction_gate": 0.6,
            "intent_classification": 0.4,
            "guardrails": 0.65
        })),
    };

    Ok(cache)
}

pub async fn update_conversation_title(
    pool: &Pool<Postgres>,
    redis: &RedisClient,
    id: Uuid,
    title: String,
) -> anyhow::Result<ConversationCache> {
    let r = sqlx::query(
        r#"
        UPDATE conversations 
        SET title = $1, updated_at = NOW() 
        WHERE id = $2 
        RETURNING id, title, user_id, soul_content, bootstrap_content, created_at, updated_at, 
                  max_token_usage, cumulative_tokens, metadata, conversation_type, gateway_thresholds
        "#
    )
    .bind(&title)
    .bind(id)
    .fetch_one(pool)
    .await?;

    invalidate_conversation_cache(redis, id).await;

    let cache = ConversationCache {
        id: r.get("id"),
        title: r.get("title"),
        user_id: r.get("user_id"),
        soul_content: r.get("soul_content"),
        bootstrap_content: r.get("bootstrap_content"),
        created_at: r.get::<Option<DateTime<Utc>>, _>("created_at").unwrap_or_else(Utc::now),
        updated_at: r.get::<Option<DateTime<Utc>>, _>("updated_at").unwrap_or_else(Utc::now),
        max_token_usage: r.get::<Option<i32>, _>("max_token_usage").unwrap_or(700000),
        cumulative_tokens: r.get::<Option<i32>, _>("cumulative_tokens").unwrap_or(0),
        conversation_type: r.get::<Option<String>, _>("conversation_type").unwrap_or_else(|| "private".to_string()),
        metadata: r.get("metadata"),
        gateway_thresholds: r.get::<Option<Value>, _>("gateway_thresholds").unwrap_or_else(|| json!({
            "interaction_gate": 0.6,
            "intent_classification": 0.4,
            "guardrails": 0.65
        })),
    };

    Ok(cache)
}

pub async fn is_conversation_member(
    pool: &Pool<Postgres>,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<bool> {
    let row = sqlx::query(
        "SELECT 1 FROM conversation_members WHERE conversation_id = $1 AND user_id = $2"
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
