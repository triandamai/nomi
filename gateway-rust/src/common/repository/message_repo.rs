use crate::feature::conversation::chat_model::MessageItem;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

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
) -> anyhow::Result<MessageItem> {
    let mut tx = pool.begin().await?;

    let save_message = sqlx::query!(
        "INSERT INTO messages (conversation_id, role, content, thought, user_id,prompt_tokens,answer_tokens,total_tokens)
         VALUES ($1, $2, $3, $4, $5,$6,$7,$8)
         RETURNING id, created_at",
        conversation_id,
        role,
        content,
        thought,
        user_id,
        prompt_tokens,
        answer_tokens,
        total_tokens,
    )
        .fetch_one(&mut *tx)
        .await;
    if let Err(err) = save_message {
        info!("Saving message failed: {}", err);
        let _ = tx.rollback().await;
        return Err(anyhow::anyhow!(err));
    };

    let row = save_message?;

    let save_convo = sqlx::query!(
        "UPDATE conversations SET cumulative_tokens = COALESCE(cumulative_tokens, 0) + $1 WHERE id = $2",
        total_tokens,
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
