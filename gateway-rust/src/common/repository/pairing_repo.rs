use anyhow::anyhow;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{rng, RngExt};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;
use crate::AppState;
use crate::feature::conversation::model::PairingResponse;

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


pub async fn create_pairing_code(
    state:&AppState,
    conversation_id: Uuid,
    user_id: Uuid,
)->anyhow::Result<PairingResponse>{


    // Verify membership
    let membership = sqlx::query!(
        "SELECT 1 as one FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        conversation_id,
        user_id
    )
        .fetch_optional(&state.pool)
        .await;

    match membership {
        Ok(Some(_)) => (),
        Ok(None) => {
            error!("User is not membership");
            return Err(anyhow!("User is not membership"))
        },
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return Err(anyhow!("Failed to verify membership"));
        }
    }

    // Generate a random 6-character alphanumeric code
    let pairing_code: String = rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    let expires_at = Utc::now() + chrono::Duration::minutes(10);

    match sqlx::query!(
        "INSERT INTO pairing_rooms (conversation_id, pairing_code, expires_at) VALUES ($1, $2, $3) RETURNING pairing_code, expires_at",
        conversation_id,
        pairing_code,
        expires_at
    )
        .fetch_one(&state.pool)
        .await
    {
        Ok(row) => {
            info!("Created pairing code");
            Ok(PairingResponse {
                pairing_code: row.pairing_code,
                expires_at: row.expires_at.unwrap_or(expires_at),
            })
        },
        Err(e) => {
            error!("Failed to create pairing room: {}", e);
            Err(anyhow!("Failed to create pairing room"))
        }
    }
}