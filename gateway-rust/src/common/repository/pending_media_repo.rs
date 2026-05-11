use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingMedia {
    pub conversation_id: Uuid,
    pub media_url: String,
    pub media_type: String,
    pub classification: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn upsert_pending_media(
    pool: &PgPool,
    conversation_id: Uuid,
    media_url: &str,
    media_type: &str,
    classification: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO pending_media (conversation_id, media_url, media_type, classification, created_at)
        VALUES ($1, $2, $3, $4, now())
        ON CONFLICT (conversation_id) DO UPDATE
        SET media_url = EXCLUDED.media_url,
            media_type = EXCLUDED.media_type,
            classification = EXCLUDED.classification,
            created_at = now()
        "#,
        conversation_id,
        media_url,
        media_type,
        classification
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_pending_media(
    pool: &PgPool,
    conversation_id: Uuid,
) -> anyhow::Result<Option<PendingMedia>> {
    let row = sqlx::query_as!(
        PendingMedia,
        r#"
        SELECT conversation_id, media_url, media_type, classification, created_at
        FROM pending_media
        WHERE conversation_id = $1
        "#,
        conversation_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn delete_pending_media(
    pool: &PgPool,
    conversation_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM pending_media WHERE conversation_id = $1",
        conversation_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
