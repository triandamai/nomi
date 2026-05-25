use crate::feature::conversation::model::{MessageItem, RepliedMessage};
use serde_json::json;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageItemWithDisplay {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub display_name: Option<String>,
    pub thought: Option<String>,
    pub image_url: Option<String>,
    pub video_url: Option<String>,
    pub audio_url: Option<String>,
    pub document_url: Option<String>,
    pub sticker_url: Option<String>,
    pub user_id: Option<Uuid>,
    pub total_tokens: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub reply_to_id: Option<Uuid>,
    pub replied_message: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MessageItemWithDisplay {
    pub fn to_sse_json(&self, token: i32) -> serde_json::Value {
        json!({
            "id": self.id,
            "conversation_id": self.conversation_id,
            "role": self.role,
            "content": self.content.clone(),
            "display_name": self.display_name,
            "thought": self.thought,
            "user_id": self.user_id,
            "total_tokens": token,
            "image_url": self.image_url.as_ref(),
            "video_url": self.video_url.as_ref(),
            "audio_url": self.audio_url.as_ref(),
            "document_url": self.document_url.as_ref(),
            "sticker_url": self.sticker_url.as_ref(),
            "metadata": self.metadata.as_ref(),
            "reply_to_id": self.reply_to_id,
            "replied_message": self.replied_message,
            "created_at": self.created_at
        })
    }
}

pub async fn mark_last_media_processed(
    pool: &sqlx::PgPool,
    conversation_id: Uuid,
) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!(
        r#"
        UPDATE conversations 
        SET metadata = metadata - 'last_image_url' - 'last_video_url' - 'last_audio_url' - 'last_doc_url' - 'last_sticker_url'
        WHERE id = $1
        RETURNING id
        "#,
        conversation_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.id))
}

pub async fn get_latest_unprocessed_media(
    pool: &sqlx::PgPool,
    conversation_id: Uuid,
) -> anyhow::Result<Option<(String, String)>> {
    let row = sqlx::query!(
        "SELECT metadata FROM conversations WHERE id = $1",
        conversation_id
    )
    .fetch_one(pool)
    .await?;

    if let Some(meta) = row.metadata {
        if let Some(url) = meta["last_image_url"].as_str() {
            return Ok(Some((url.to_string(), "image".to_string())));
        }
        if let Some(url) = meta["last_video_url"].as_str() {
            return Ok(Some((url.to_string(), "video".to_string())));
        }
        if let Some(url) = meta["last_audio_url"].as_str() {
            return Ok(Some((url.to_string(), "audio".to_string())));
        }
        if let Some(url) = meta["last_doc_url"].as_str() {
            return Ok(Some((url.to_string(), "document".to_string())));
        }
        if let Some(url) = meta["last_sticker_url"].as_str() {
            return Ok(Some((url.to_string(), "sticker".to_string())));
        }
    }

    Ok(None)
}

pub async fn save_message(
    pool: &sqlx::PgPool,
    conversation_id: uuid::Uuid,
    role: &str,
    content: &str,
    thought: Option<&str>,
    user_id: Option<uuid::Uuid>,
    p_tokens: i32,
    a_tokens: i32,
    t_tokens: i32,
    image_url: Option<String>,
    video_url: Option<String>,
    audio_url: Option<String>,
    document_url: Option<String>,
    sticker_url: Option<String>,
    metadata: Option<serde_json::Value>,
    reply_to_id: Option<uuid::Uuid>,
    redis: Option<&crate::common::redis::RedisClient>,
) -> anyhow::Result<MessageItemWithDisplay> {
    info!(
        "Saving message to conversation:{:?} from user:{:?} reply_to:{:?}",
        conversation_id, user_id, reply_to_id
    );
    let mut tx = pool.begin().await?;

    // 🌟 SRP REFINEMENT: Explicitly join replied message in the RETURNING clause or handle separately
    // Since SQLx query! doesn't support complex JOINs in RETURNING, we use raw query and fetch the joined data
    let res = sqlx::query(
        "WITH inserted AS (
            INSERT INTO messages (conversation_id, role, content, thought, user_id, prompt_tokens, answer_tokens, total_tokens, image_url, video_url, audio_url, document_url, sticker_url, metadata, reply_to_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id, created_at, metadata, reply_to_id, conversation_id, role, content, thought, user_id, total_tokens, image_url, video_url, audio_url, document_url, sticker_url
        )
        SELECT 
            i.*,
            CASE WHEN i.reply_to_id IS NOT NULL THEN
                 (SELECT jsonb_build_object(
                   'id', rm.id,
                   'role', rm.role,
                   'content', rm.content,
                   'display_name', ru.display_name
                 ) FROM messages rm LEFT JOIN users ru ON ru.id = rm.user_id WHERE rm.id = i.reply_to_id)
            ELSE NULL END as replied_message
        FROM inserted i"
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(thought)
    .bind(user_id)
    .bind(p_tokens)
    .bind(a_tokens)
    .bind(t_tokens)
    .bind(image_url.clone())
    .bind(video_url.clone())
    .bind(audio_url.clone())
    .bind(document_url.clone())
    .bind(sticker_url.clone())
    .bind(metadata.clone())
    .bind(reply_to_id)
    .fetch_one(&mut *tx)
    .await?;

    use sqlx::Row;
    let row_id: Uuid = res.get("id");
    let row_created_at: chrono::DateTime<chrono::Utc> = res.get("created_at");
    let row_metadata: Option<serde_json::Value> = res.get("metadata");
    let row_reply_to_id: Option<Uuid> = res.get("reply_to_id");
    let row_replied_message: Option<serde_json::Value> = res.get("replied_message");

    let meta_update = json!({
        "last_image_url":image_url,
        "last_video_url":video_url,
        "last_audio_url":audio_url,
        "last_doc_url":document_url,
        "last_sticker_url":sticker_url
    });
    
    let save_convo = sqlx::query!(
        "UPDATE conversations SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1, metadata = COALESCE(metadata, '{}'::jsonb) || $2 WHERE id = $3",
        t_tokens,
        meta_update,
        conversation_id
    )
        .execute(&mut *tx)
        .await;

    if let Err(err) = save_convo {
        info!("Saving message failed: {}", err);
        let _ = tx.rollback().await;
        return Err(anyhow::anyhow!(err));
    };

    match tx.commit().await {
        Ok(_) => {
            info!("Successfully saved message");

            // Update Cache tokens if redis is provided
            if let Some(r) = redis {
                // Fetch the new total tokens after the update to be precise
                let total_updated: i32 = sqlx::query_scalar!(
                    "SELECT cumulative_tokens FROM conversations WHERE id = $1",
                    conversation_id
                )
                .fetch_one(pool)
                .await
                .unwrap_or(Some(0))
                .unwrap_or(0);

                crate::common::repository::conversation_repo::update_cached_tokens(
                    r,
                    conversation_id,
                    total_updated
                ).await;
            }

            // Telemetry
            let pool_clone = pool.clone();
            let conv_id = conversation_id.clone();
            let msg_id = row_id.clone();
            let u_id = user_id.clone();
            let role_clone = role.to_string();
            let total_tokens_val = t_tokens as i64;
            let prompt_tokens_val = p_tokens as i64;
            let answer_tokens_val = a_tokens as i64;

            tokio::spawn(async move {
                let _ = crate::services::ambient_soul::AmbientSoulService::log_token_transaction(
                    &pool_clone,
                    Some(conv_id),
                    Some(msg_id),
                    u_id,
                    "message",
                    &role_clone,
                    prompt_tokens_val,
                    answer_tokens_val,
                    total_tokens_val,
                )
                .await;
            });

            Ok(MessageItemWithDisplay {
                id: row_id,
                conversation_id,
                role: role.to_string(),
                content: content.to_string(),
                display_name: None,
                thought: thought.map(|s| s.to_string()),
                image_url,
                video_url,
                audio_url,
                document_url,
                sticker_url,
                user_id,
                total_tokens: Some(t_tokens),
                metadata: row_metadata,
                reply_to_id: row_reply_to_id,
                replied_message: row_replied_message,
                created_at: row_created_at,
            })
        }
        Err(err) => {
            info!("Saving message failed: {}", err);
            Err(anyhow::anyhow!(err))
        }
    }
}

impl From<MessageItemWithDisplay> for MessageItem {
    fn from(m: MessageItemWithDisplay) -> Self {
        let replied_msg = m.replied_message.and_then(|v| serde_json::from_value::<RepliedMessage>(v).ok());
        Self {
            id: m.id,
            conversation_id: m.conversation_id,
            display_name: m.display_name,
            role: m.role,
            content: m.content,
            total_tokens: m.total_tokens,
            answer_tokens: None, 
            prompt_tokens: None,
            thought: m.thought,
            image_url: m.image_url,
            video_url: m.video_url,
            audio_url: m.audio_url,
            document_url: m.document_url,
            sticker_url: m.sticker_url,
            user_id: m.user_id,
            created_at: m.created_at,
            metadata: m.metadata,
            reply_to_id: m.reply_to_id,
            replied_message: replied_msg,
        }
    }
}
