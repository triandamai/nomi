use crate::AppState;
use crate::prompts::PromptRegistry;
use log::info;
use tracing::error;
use uuid::Uuid;

pub mod auth_model;
pub mod middleware;

pub struct UserIdentity {
    pub id: Uuid,
    pub display_name: String,
}

pub async fn resolve_identity(
    state: &AppState,
    external_sender_id: &str,
    external_chat_id: &str,
    channel_type: &str,
    is_group: bool,
    display_name: String,
) -> anyhow::Result<UserIdentity> {
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return Err(anyhow::anyhow!("Db trx failed"));
        }
    };
    // Basic resolution: upsert user by external_id
    if is_group {
        info!("User from group, check existing, if user doesnt exist create new one external_sender_id:{} external_chat_id:{} channel:{}",external_sender_id,external_chat_id,channel_type);
        let existing_channel_of_user = sqlx::query!(
            "SELECT c.user_id FROM channels c WHERE c.external_chat_id = $1 AND c.channel_type = $2",
            external_sender_id,
            channel_type
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = existing_channel_of_user {
            info!("User doesnt exist, since user from group, we create user directly, err:{}",e);
            let save_new_user = sqlx::query!(
                "INSERT INTO users (external_id, display_name)
                VALUES ($1, $2)
                ON CONFLICT (external_id)
                DO UPDATE SET display_name = EXCLUDED.display_name
                RETURNING id, display_name",
                external_sender_id,
                display_name
            )
            .fetch_one(&mut *tx)
            .await;

            if let Err(e) = save_new_user {
                info!(
                    "User doesnt exist, since user from group create new one, but failed:{}",
                    e
                );
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!(
                    "User doesnt exist, since user from group create new one, but attempt failed"
                ));
            }
            let new_user = save_new_user.unwrap();

            let save_new_conv = sqlx::query!(
                "INSERT INTO conversations (title,soul_content,bootstrap_content,user_id,conversation_type)
                VALUES ($1, $2,$3,$4,$5) RETURNING id",
                display_name,
                PromptRegistry::default_soul_prompts(),
                PromptRegistry::default_rules_prompts(),
                new_user.id,
                "private"
            )
                .fetch_one(&mut *tx)
                .await;
            if let Err(e) = save_new_conv {
                info!(
                    "User Convo doesnt exist, since user from group create new one, but failed:{}",
                    e
                );
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!(
                    "User Convo doesnt exist, since user from group create new one, but attempt failed"
                ));
            }
            let new_convo = save_new_conv.unwrap();
            let linked_channels = sqlx::query!(
                "INSERT INTO channels(channel_type,external_id,external_chat_id, conversation_id,user_id,created_at)
                 VALUES ($1,$2,$3,$4,$5,now()) RETURNING id
                ",
                channel_type,
                external_sender_id,
                external_sender_id,
                new_convo.id,
                new_user.id
            )
            .fetch_one(&mut *tx)
            .await;

            if let Err(e) = linked_channels {
                info!(
                    "User Channels doesnt exist, since user from group create new one, but failed:{}",
                    e
                );
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!(
                    "User Channels doesnt exist, since user from group create new one, but attempt failed"
                ));
            }

            if let Err(err) = tx.commit().await{
                info!("Db trx commit failed {}",err);
                return Err(anyhow::anyhow!("Failed to commit transaction: {}", err));
            }


            return Ok(UserIdentity {
                id: new_user.id,
                display_name,
            });
        }
    }

    let row = sqlx::query!(
        "INSERT INTO users (external_id, display_name) 
         VALUES ($1, $2) 
         ON CONFLICT (external_id) 
         DO UPDATE SET display_name = EXCLUDED.display_name 
         RETURNING id, display_name",
        external_chat_id,
        display_name
    )
    .fetch_one(&mut *tx)
    .await;

    if let Err(e) = row {
        info!("Failed getting identity:{}", e);
        let _ = tx.rollback().await;
        return Err(anyhow::anyhow!("Failed getting identity:"));
    }
    let row = row.unwrap();
    Ok(UserIdentity {
        id: row.id,
        display_name: row
            .display_name
            .unwrap_or_else(|| external_chat_id.to_string()),
    })
}
