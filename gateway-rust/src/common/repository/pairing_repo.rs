use anyhow::anyhow;
use chrono::Utc;
use rand::distr::Alphanumeric;
use rand::{rng, RngExt};
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;
use crate::AppState;
use crate::feature::conversation::model::PairingResponse;
use serde_json::json;

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
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
) -> anyhow::Result<PairingResponse> {
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
            return Err(anyhow!("User is not membership"));
        }
        Err(e) => {
            error!("Failed to verify membership: {}", e);
            return Err(anyhow!("Failed to verify membership"));
        }
    }

    // Generate a secure 6-digit code string formatted as XXX-XXX
    let code_part1: String = rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    let code_part2: String = rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect::<String>()
        .to_uppercase();

    let pairing_code = format!("{}-{}", code_part1, code_part2);
    let expires_at = Utc::now() + chrono::Duration::minutes(5);

    // Save to Redis with 5-minute TTL
    let redis_key = format!("pairing:{}", pairing_code);
    let payload = json!({
        "user_id": user_id,
        "conversation_id": conversation_id
    });
    
    state
        .redis
        .set_ex(&redis_key, &payload.to_string(), 300)
        .await?;

    info!("Created Redis-backed pairing code: {}", pairing_code);

    Ok(PairingResponse {
        pairing_code,
        expires_at,
    })
}