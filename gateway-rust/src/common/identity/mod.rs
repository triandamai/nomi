use uuid::Uuid;
use sqlx::PgPool;

pub struct UserIdentity {
    pub id: Uuid,
    pub display_name: String,
}

pub async fn resolve_identity(
    pool: &PgPool,
    external_id: &str,
    _channel_type: &str,
) -> anyhow::Result<UserIdentity> {
    // Basic resolution: upsert user by external_id
    let row = sqlx::query!(
        "INSERT INTO users (external_id, display_name) 
         VALUES ($1, $2) 
         ON CONFLICT (external_id) 
         DO UPDATE SET display_name = EXCLUDED.display_name 
         RETURNING id, display_name",
        external_id,
        external_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserIdentity {
        id: row.id,
        display_name: row.display_name.unwrap_or_else(|| external_id.to_string()),
    })
}
