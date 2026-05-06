use sqlx::PgPool;
use uuid::Uuid;

pub struct ChannelInfo {
    pub user_id: Uuid,
    pub conversation_id: Uuid,
}

pub async fn get_channel_info(
    pool: &PgPool,
    channel_type: &str,
    external_chat_id: &str,
) -> anyhow::Result<Option<ChannelInfo>> {
    let row = sqlx::query!(
        "SELECT user_id, conversation_id FROM channels 
         WHERE channel_type = $1 AND external_chat_id = $2",
        channel_type,
        external_chat_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        if let (Some(u), Some(c)) = (r.user_id, r.conversation_id) {
            return Ok(Some(ChannelInfo {
                user_id: u,
                conversation_id: c,
            }));
        }
    }
    Ok(None)
}

pub async fn link_channel(
    pool: &PgPool,
    channel_type: &str,
    external_id: &str,
    external_chat_id: &str,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO channels (channel_type, external_id, external_chat_id, conversation_id, user_id) 
         VALUES ($1, $2, $3, $4, $5) 
         ON CONFLICT (channel_type, external_chat_id) 
         DO UPDATE SET conversation_id = EXCLUDED.conversation_id, user_id = EXCLUDED.user_id",
        channel_type,
        external_id,
        external_chat_id,
        conversation_id,
        user_id
    )
    .execute(pool)
    .await?;

    // Also ensure they are conversation members
    sqlx::query!(
        "INSERT INTO conversation_members (conversation_id, user_id) 
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
        conversation_id,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
