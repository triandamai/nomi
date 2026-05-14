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
    // Split by ':' to get the prefix, and by '@' to get the domain

    if external_sender_id.contains(":") {
        info!(
            "External sender id: {} cannot contains :",
            external_sender_id
        );
        return Err(anyhow::anyhow!("External sender_id contains ':'"));
    }
    // Basic resolution: upsert user by external_id
    if is_group {
        info!(
            "User from group, check existing, if user doesnt exist create new one external_sender_id:{} external_chat_id:{} channel:{}",
            external_sender_id, external_chat_id, channel_type
        );
        let existing_channel_of_user = sqlx::query!(
            "
            SELECT c.user_id, u.display_name  FROM channels as c
            RIGHT JOIN users as u ON u.id = c.user_id
            WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            channel_type,
            external_sender_id
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = existing_channel_of_user {
            info!("User Channel doesnt exist create new one, but error:{}", e);
            let save_new_user = sqlx::query!(
                "
                INSERT INTO users (display_name)
                VALUES ($1) RETURNING id, display_name",
                display_name
            )
            .fetch_one(&mut *tx)
            .await;

            if let Err(e) = save_new_user {
                info!("Create User in group failed:{}", e);
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!("Create User in group failed"));
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

            if let Err(err) = tx.commit().await {
                info!("Db trx commit failed {}", err);
                return Err(anyhow::anyhow!("Failed to commit transaction: {}", err));
            }

            return Ok(UserIdentity {
                id: new_user.id,
                display_name,
            });
        }

        let row = existing_channel_of_user.unwrap();
        if let None = row.user_id {
            info!("Record not exist, user doesnt exist");
            return Err(anyhow::anyhow!("User doesnt exist"));
        }
        if let Err(e) = tx.commit().await {
            info!("Db trx commit failed {}", e);
            return Err(anyhow::anyhow!("Failed to commit transaction: {}", e));
        }
        let id = row.user_id.unwrap();
        Ok(UserIdentity {
            id,
            display_name: row.display_name.unwrap_or_else(|| display_name.to_string()),
        })
    } else {
        let find_user = sqlx::query!(
            "
            SELECT c.user_id, u.display_name  FROM channels as c
            RIGHT JOIN users as u ON u.id = c.user_id
            WHERE c.channel_type = $1 AND c.external_chat_id = $2",
            channel_type,
            external_sender_id
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = find_user {
            info!("failed getting information identity private dm:{}", e);
            let _ = tx.rollback().await;
            return Err(anyhow::anyhow!("Failed getting information identity:"));
        }

        if let Err(e) = tx.commit().await {
            info!("Db trx commit failed:{}", e);
            return Err(anyhow::anyhow!("Failed to commit transaction"));
        }
        let row = find_user?;
        if let None = row.user_id {
            info!("User doesnt exist");
            return Err(anyhow::anyhow!("User not found"));
        }
        let id = row.user_id.unwrap();
        Ok(UserIdentity {
            id,
            display_name: row.display_name.unwrap_or_else(|| display_name.to_string()),
        })
    }
}
