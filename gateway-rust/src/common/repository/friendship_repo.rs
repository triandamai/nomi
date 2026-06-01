use sqlx::{Pool, Postgres};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::anyhow;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendProfile {
    pub id: Uuid,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FriendRequestItem {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub sender_display_name: Option<String>,
    pub receiver_display_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn send_friend_request(
    pool: &Pool<Postgres>,
    sender_id: Uuid,
    receiver_id: Uuid,
) -> anyhow::Result<()> {
    if sender_id == receiver_id {
        return Err(anyhow!("You cannot send a friend request to yourself"));
    }

    // Check if they are already friends
    let u1 = std::cmp::min(sender_id, receiver_id);
    let u2 = std::cmp::max(sender_id, receiver_id);
    let is_friend = sqlx::query!(
        "SELECT 1 as one FROM friendships WHERE user_id_1 = $1 AND user_id_2 = $2",
        u1,
        u2
    )
    .fetch_optional(pool)
    .await?;

    if is_friend.is_some() {
        return Err(anyhow!("You are already friends with this user"));
    }

    // Check if a block exists in either direction
    let is_blocked = sqlx::query!(
        "SELECT 1 as one FROM blocked_users WHERE (user_id = $1 AND blocked_user_id = $2) OR (user_id = $2 AND blocked_user_id = $1)",
        sender_id,
        receiver_id
    )
    .fetch_optional(pool)
    .await?;

    if is_blocked.is_some() {
        return Err(anyhow!("Cannot send friend request: block exists"));
    }

    // Insert friend request
    sqlx::query!(
        "INSERT INTO friend_requests (sender_id, receiver_id) VALUES ($1, $2) ON CONFLICT (sender_id, receiver_id) DO NOTHING",
        sender_id,
        receiver_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn respond_friend_request(
    pool: &Pool<Postgres>,
    receiver_id: Uuid,
    sender_id: Uuid,
    accept: bool,
) -> anyhow::Result<Option<Uuid>> {
    let mut tx = pool.begin().await?;

    // Verify request exists
    let req = sqlx::query!(
        "SELECT id FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2",
        sender_id,
        receiver_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    if req.is_none() {
        return Err(anyhow!("Friend request not found"));
    }

    // Delete the request
    sqlx::query!(
        "DELETE FROM friend_requests WHERE sender_id = $1 AND receiver_id = $2",
        sender_id,
        receiver_id
    )
    .execute(&mut *tx)
    .await?;

    if !accept {
        tx.commit().await?;
        return Ok(None);
    }

    // Insert friendship with deterministic UUID order
    let u1 = std::cmp::min(sender_id, receiver_id);
    let u2 = std::cmp::max(sender_id, receiver_id);

    sqlx::query!(
        "INSERT INTO friendships (user_id_1, user_id_2) VALUES ($1, $2) ON CONFLICT (user_id_1, user_id_2) DO NOTHING",
        u1,
        u2
    )
    .execute(&mut *tx)
    .await?;

    // Auto-provision a unique private DM conversation channel
    let conversation_id = Uuid::new_v4();

    // Fetch display names to build conversation title
    let sender_name = sqlx::query!("SELECT display_name FROM users WHERE id = $1", sender_id)
        .fetch_one(&mut *tx)
        .await?
        .display_name
        .unwrap_or_else(|| "User A".to_string());

    let receiver_name = sqlx::query!("SELECT display_name FROM users WHERE id = $1", receiver_id)
        .fetch_one(&mut *tx)
        .await?
        .display_name
        .unwrap_or_else(|| "User B".to_string());

    let title = format!("{} & {}", sender_name, receiver_name);

    sqlx::query!(
        "INSERT INTO conversations (id, title, conversation_type) VALUES ($1, $2, 'private')",
        conversation_id,
        title
    )
    .execute(&mut *tx)
    .await?;

    // Add both members
    sqlx::query!(
        "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2), ($1, $3)",
        conversation_id,
        sender_id,
        receiver_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Some(conversation_id))
}

pub async fn get_friends(
    pool: &Pool<Postgres>,
    user_id: Uuid,
) -> anyhow::Result<Vec<FriendProfile>> {
    let rows = sqlx::query_as!(
        FriendProfile,
        r#"
        SELECT u.id, u.name, u.display_name, u.email
        FROM friendships f
        JOIN users u ON (f.user_id_1 = u.id AND f.user_id_2 = $1) OR (f.user_id_2 = u.id AND f.user_id_1 = $1)
        ORDER BY COALESCE(u.display_name, u.name) ASC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_pending_requests(
    pool: &Pool<Postgres>,
    user_id: Uuid,
) -> anyhow::Result<(Vec<FriendRequestItem>, Vec<FriendRequestItem>)> {
    let incoming = sqlx::query_as!(
        FriendRequestItem,
        r#"
        SELECT fr.id, fr.sender_id, fr.receiver_id, 
               u1.display_name as sender_display_name, 
               u2.display_name as receiver_display_name, 
               fr.created_at
        FROM friend_requests fr
        JOIN users u1 ON fr.sender_id = u1.id
        JOIN users u2 ON fr.receiver_id = u2.id
        WHERE fr.receiver_id = $1
        ORDER BY fr.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    let outgoing = sqlx::query_as!(
        FriendRequestItem,
        r#"
        SELECT fr.id, fr.sender_id, fr.receiver_id, 
               u1.display_name as sender_display_name, 
               u2.display_name as receiver_display_name, 
               fr.created_at
        FROM friend_requests fr
        JOIN users u1 ON fr.sender_id = u1.id
        JOIN users u2 ON fr.receiver_id = u2.id
        WHERE fr.sender_id = $1
        ORDER BY fr.created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok((incoming, outgoing))
}

pub async fn block_user(
    pool: &Pool<Postgres>,
    user_id: Uuid,
    blocked_user_id: Uuid,
) -> anyhow::Result<()> {
    let mut tx = pool.begin().await?;

    // Insert block entry
    sqlx::query!(
        "INSERT INTO blocked_users (user_id, blocked_user_id) VALUES ($1, $2) ON CONFLICT (user_id, blocked_user_id) DO NOTHING",
        user_id,
        blocked_user_id
    )
    .execute(&mut *tx)
    .await?;

    // Clean up friendships if existing
    let u1 = std::cmp::min(user_id, blocked_user_id);
    let u2 = std::cmp::max(user_id, blocked_user_id);
    sqlx::query!(
        "DELETE FROM friendships WHERE user_id_1 = $1 AND user_id_2 = $2",
        u1,
        u2
    )
    .execute(&mut *tx)
    .await?;

    // Clean up any pending friend requests
    sqlx::query!(
        "DELETE FROM friend_requests WHERE (sender_id = $1 AND receiver_id = $2) OR (sender_id = $2 AND receiver_id = $1)",
        user_id,
        blocked_user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
