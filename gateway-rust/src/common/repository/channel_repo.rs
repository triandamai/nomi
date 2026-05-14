use rust_decimal::prelude::ToPrimitive;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ChannelInfo {
    pub user_id: Uuid,
    pub conversation_id: Uuid,
    pub cumulative_tokens: i64,
    pub max_token_usage: f64,
}

pub struct ChannelGroupInfo {
    pub conversation_id: Uuid,
    pub cumulative_tokens: i64,
    pub max_token_usage: f64,
}

pub async fn get_channel_info(
    pool: &PgPool,
    channel_type: &str,
    external_chat_id: &str,
) -> anyhow::Result<Option<ChannelInfo>> {
    let row = sqlx::query!(
        "SELECT channels.user_id, channels.conversation_id, conversations.cumulative_tokens, conversations.max_token_usage FROM channels
         JOIN conversations ON conversations.id = channels.conversation_id
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
                cumulative_tokens: r.cumulative_tokens.map_or_else(||0, |v|v.to_i64().unwrap_or(0)),
                max_token_usage: r.max_token_usage.map_or_else(|| 700000.0, |v| v.to_f64().unwrap_or(700000.0)),
            }));
        }
    }
    Ok(None)
}

pub async fn get_channel_group_info(
    pool: &PgPool,
    channel: &str,
    external_chat_id: &str,
) -> anyhow::Result<Option<ChannelGroupInfo>> {
    let row = sqlx::query!(
        "SELECT channel_group.conversation_id, conversations.cumulative_tokens, conversations.max_token_usage FROM channel_group
         JOIN conversations ON conversations.id = channel_group.conversation_id
         WHERE channel = $1 AND external_group_id = $2",
        channel,
        external_chat_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = row {
        return Ok(Some(ChannelGroupInfo {
            conversation_id: r.conversation_id,
            cumulative_tokens: r
                .cumulative_tokens
                .map_or_else(|| 0, |v| v.to_i64().unwrap_or(0)),
            max_token_usage: r.max_token_usage.map_or_else(|| 700000.0, |v| v.to_f64().unwrap_or(700000.0)),
        }));
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
    display_name: Option<String>,
) -> anyhow::Result<()> {
    let _ = sqlx::query!(
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
    .await;

    // Also ensure they are conversation members
    let _ = sqlx::query!(
        "INSERT INTO conversation_members (conversation_id, user_id) 
         VALUES ($1, $2) ON CONFLICT DO NOTHING",
        conversation_id,
        user_id
    )
    .execute(pool)
    .await;

    if let Some(display_name) = display_name {
        let _ = sqlx::query!(
            "UPDATE users  SET display_name = $1 WHERE id = $2",
            display_name,
            user_id
        )
        .execute(pool)
        .await;
    }
    Ok(())
}

pub async fn link_channel_group(
    pool: &PgPool,
    channel: &str,
    external_group_id: &str,
    conversation_id: Uuid,
) -> anyhow::Result<()> {
    let _ = sqlx::query!(
        "INSERT INTO channel_group (conversation_id, channel, external_group_id, registered_at,is_active)
         VALUES ($1, $2, $3, now(), true)",
        conversation_id,
        channel,
        external_group_id
    ).execute(pool).await;

    Ok(())
}

pub async fn is_group_registered(pool: &sqlx::PgPool, external_id: &str, channel: &str) -> bool {
    sqlx::query!(
        "SELECT is_active FROM channel_group WHERE external_group_id = $1 AND channel = $2",
        external_id,
        channel
    )
    .fetch_optional(pool)
    .await
    .map(|r| r.map(|row| row.is_active.unwrap_or(true)).unwrap_or(false))
    .unwrap_or(false)
}
