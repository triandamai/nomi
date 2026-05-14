use crate::feature::conversation::model::MessageItem;
use serde_json::json;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

pub async fn mark_last_media_processed(pool: &PgPool, conversation_id: Uuid) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "UPDATE messages 
         SET metadata = jsonb_set(COALESCE(metadata, '{}'::jsonb), '{is_processed}', 'true') 
         WHERE id = (
             SELECT id FROM messages 
             WHERE conversation_id = $1 
             AND (image_url IS NOT NULL OR video_url IS NOT NULL OR audio_url IS NOT NULL OR document_url IS NOT NULL OR sticker_url IS NOT NULL)
             ORDER BY created_at DESC 
             LIMIT 1
         )",
        conversation_id
    )
    .execute(&mut *tx)
    .await?;

    // Also clear from pending_media table
    sqlx::query!(
        "DELETE FROM pending_media WHERE conversation_id = $1",
        conversation_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn save_message(
    pool: &PgPool,
    conversation_id: Uuid,
    role: &str,
    content: &str,
    thought: Option<&str>,
    user_id: Option<Uuid>,
    prompt_tokens: i32,
    answer_tokens: i32,
    total_tokens: i32,
    image_url: Option<String>,
    video_url: Option<String>,
    audio_url: Option<String>,
    document_url: Option<String>,
    sticker_url: Option<String>
) -> anyhow::Result<MessageItem> {
    info!("Saving message to conversation:{:?} from user:{:?}",conversation_id,user_id);
    let mut tx = pool.begin().await?;

    let save_message = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id, prompt_tokens, answer_tokens, total_tokens, image_url, video_url, audio_url, document_url, sticker_url)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING id, created_at",
        conversation_id,
        role,
        content,
        thought,
        user_id,
        prompt_tokens,
        answer_tokens,
        total_tokens,
        image_url,
        video_url,
        audio_url,
        document_url,
        sticker_url
    )
        .fetch_one(&mut *tx)
        .await;
    if let Err(err) = save_message {
        info!("Saving message failed: {}", err);
        let _ = tx.rollback().await;
        return Err(anyhow::anyhow!(err));
    };

    let row = save_message?;

    let meta = Some(json!({
        "last_image_url":image_url,
        "last_video_url":video_url,
        "last_audio_url":audio_url,
        "last_doc_url":document_url,
        "last_sticker_url":sticker_url
    }));
    let save_convo = sqlx::query!(
        "UPDATE conversations SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1, metadata=$2 WHERE id = $3",
        total_tokens,
        meta,
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
            Ok(MessageItem {
                id: row.id,
                conversation_id,
                role: role.to_string(),
                content: content.to_string(),
                thought: thought.map(|s| s.to_string()),
                total_tokens: Some(total_tokens),
                prompt_tokens: Some(prompt_tokens),
                answer_tokens: Some(answer_tokens),
                image_url,
                video_url,
                audio_url,
                document_url,
                sticker_url,
                user_id,
                created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
            })
        }
        Err(err) => {
            info!("Saving message failed: {}", err);
            Err(anyhow::anyhow!(err))
        }
    }
}
