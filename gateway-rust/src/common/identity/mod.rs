use crate::AppState;
use sqlx::{Pool, Postgres};
use crate::prompts::PromptRegistry;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use tracing::error;
use uuid::Uuid;

pub mod auth_model;
pub mod middleware;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserIdentity {
    pub id: Uuid,
    pub display_name: String,
}

impl Display for UserIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&serde_json::to_string(&self).unwrap())
    }
}

pub async fn resolve_identity_by_id(pool: &Pool<Postgres>, user_id: Uuid) -> anyhow::Result<UserIdentity> {
    let row = sqlx::query!(
        "SELECT id, display_name FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(UserIdentity {
        id: row.id,
        display_name: row.display_name.unwrap_or_else(|| "User".to_string()),
    })
}

pub async fn resolve_identity(
    state: &AppState,
    external_sender_id: &str,
    external_chat_id: &str,
    channel_type: &str,
    is_group: bool,
    display_name: String,
) -> anyhow::Result<UserIdentity> {
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
            "User from group, create new one if not exist external_sender_id:{} external_chat_id:{} channel:{}",
            external_sender_id, external_chat_id, channel_type
        );
        let mut tx = match state.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return Err(anyhow::anyhow!("Db trx failed"));
            }
        };

        // 🌟 SRP REFINEMENT: Lookup by external_id (stable ID like LID)
        let existing_channel_of_user = sqlx::query!(
            "
            SELECT c.user_id, u.display_name FROM channels as c
            RIGHT JOIN users as u ON u.id = c.user_id
            WHERE c.channel_type = $1 AND (c.external_id = $2 OR c.external_chat_id = $2)",
            channel_type,
            external_sender_id
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = existing_channel_of_user {
            info!("Failed getting existing channels:{}", e);
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
                info!("Create convo failed:{}", e);
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!("Create User in group failed"));
            }
            let new_convo = save_new_conv?;
            let save_members = sqlx::query!(
                "INSERT INTO conversation_members (conversation_id,user_id,joined_at)
                VALUES ($1,$2,now()) RETURNING conversation_id,user_id",
                new_convo.id,
                new_user.id
            )
            .fetch_one(&mut *tx)
            .await;

            if let Err(e) = save_members {
                info!("Failed create members:{}", e);
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!("Faield create member :{e}"));
            }
            
            // 🌟 SRP REFINEMENT: Save both stable ID and reachable chat ID
            let linked_channels = sqlx::query!(
                "INSERT INTO channels(channel_type,external_id,external_chat_id, conversation_id,user_id,created_at)
                 VALUES ($1,$2,$3,$4,$5,now()) 
                 ON CONFLICT (channel_type, external_chat_id) 
                 DO UPDATE SET external_id = EXCLUDED.external_id, user_id = EXCLUDED.user_id
                 RETURNING id
                ",
                channel_type,
                external_sender_id, // Stable LID/JID (save to external_id)
                external_chat_id,   // Reachable chat ID (save to external_chat_id)
                new_convo.id,
                new_user.id
            )
            .fetch_one(&mut *tx)
            .await;

            if let Err(e) = linked_channels {
                info!("Failed linking channels:{}", e);
                let _ = tx.rollback().await;
                return Err(anyhow::anyhow!("Failed linking channels"));
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

        let row = existing_channel_of_user?;
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
        let mut tx = match state.pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return Err(anyhow::anyhow!("Db trx failed"));
            }
        };

        // 🌟 SRP REFINEMENT: Lookup by external_id (LID) or external_chat_id (Phone)
        let find_user = sqlx::query!(
            "
            SELECT c.user_id, u.display_name FROM channels as c
            RIGHT JOIN users as u ON u.id = c.user_id
            WHERE c.channel_type = $1 AND (c.external_id = $2 OR c.external_chat_id = $3)",
            channel_type,
            external_sender_id,
            external_chat_id
        )
        .fetch_one(&mut *tx)
        .await;

        if let Err(e) = find_user {
            info!("failed getting information identity private dm:{}", e);
            let _ = tx.rollback().await;
            return Err(anyhow::anyhow!("Failed getting information identity:"));
        }
        
        let row = find_user.unwrap();
        
        // 🌟 SRP REFINEMENT: Ensure external_chat_id is updated if it differs (e.g. phone changed)
        if let Some(uid) = row.user_id {
            let _ = sqlx::query!(
                "UPDATE channels SET external_chat_id = $1, external_id = $2 WHERE channel_type = $3 AND user_id = $4",
                external_chat_id,
                external_sender_id,
                channel_type,
                uid
            )
            .execute(&mut *tx)
            .await;
        }

        if let Err(e) = tx.commit().await {
            info!("Db trx commit failed:{}", e);
            return Err(anyhow::anyhow!("Failed to commit transaction"));
        }

        //======//
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
