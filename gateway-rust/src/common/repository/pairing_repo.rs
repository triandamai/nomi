use sqlx::PgPool;
use uuid::Uuid;

pub async fn validate_pairing_code(
    pool: &PgPool,
    code: &str,
) -> anyhow::Result<Option<Uuid>> {
    let row = sqlx::query!(
        "SELECT conversation_id FROM pairing_rooms 
         WHERE pairing_code = $1 AND expires_at > now() AND user_id IS NULL",
        code
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.conversation_id))
}

pub async fn complete_pairing(
    pool: &PgPool,
    code: &str,
    user_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE pairing_rooms SET user_id = $1 
         WHERE pairing_code = $2 AND expires_at > now()",
        user_id,
        code
    )
    .execute(pool)
    .await?;
    Ok(())
}
