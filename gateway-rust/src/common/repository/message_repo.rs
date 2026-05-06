use sqlx::PgPool;
use uuid::Uuid;
use crate::feature::conversation::chat_model::MessageItem;

pub async fn save_message(
    pool: &PgPool,
    conversation_id: Uuid,
    role: &str,
    content: &str,
    thought: Option<&str>,
    user_id: Option<Uuid>,
) -> anyhow::Result<MessageItem> {
    let row = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id) 
         VALUES ($1, $2, $3, $4, $5) 
         RETURNING id, created_at",
        conversation_id,
        role,
        content,
        thought,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(MessageItem {
        id: row.id,
        conversation_id,
        role: role.to_string(),
        content: content.to_string(),
        thought: thought.map(|s| s.to_string()),
        user_id,
        created_at: row.created_at.unwrap_or_else(chrono::Utc::now),
    })
}
