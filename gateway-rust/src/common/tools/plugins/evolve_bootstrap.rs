use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, EvolveBootstrapParameters};
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;

pub struct EvolveBootstrapPlugin;

impl NomiToolPlugin for EvolveBootstrapPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "evolve_bootstrap_content",
                "Update your own personality or mission instructions (System Prompt) dynamically.",
                None,
            )
            .with_parameters::<EvolveBootstrapParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["EVOLVE_BOOTSTRAP", "UPDATE_SYSTEM_PROMPT", "CHANGE_CORE_INSTRUCTIONS", "DASHBOARD", "GENERAL"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: EvolveBootstrapParameters = serde_json::from_value(args)?;
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok(ToolResult {
                        error: "No active conversation to evolve.".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let result: Result<Option<i32>, sqlx::Error> = async {
                let mut tx = dispatcher.pool.begin().await?;

                let convo = sqlx::query!(
                    "SELECT soul_content, bootstrap_content FROM conversations WHERE id = $1 FOR UPDATE",
                    conversation_id
                )
                .fetch_one(&mut *tx)
                .await?;

                let next_version: i32 = sqlx::query_scalar(
                    "SELECT (COALESCE(MAX(version_number), 0) + 1)::INT4 FROM soul_history WHERE conversation_id = $1",
                )
                .bind(conversation_id)
                .fetch_one(&mut *tx)
                .await?;

                sqlx::query(
                    "UPDATE conversations SET bootstrap_content = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(&params.updated_instructions)
                .bind(conversation_id)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO soul_history (conversation_id, soul_content, bootstrap, change_reason, version_number) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(conversation_id)
                .bind(convo.soul_content)
                .bind(&params.updated_instructions)
                .bind(&params.reason)
                .bind(next_version)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;

                // Invalidate Cache
                crate::common::repository::conversation_repo::invalidate_conversation_cache(
                    &dispatcher.app_state.redis,
                    conversation_id
                ).await;

                Ok(Some(next_version))
            }
            .await;

            match result {
                Ok(Some(version)) => {
                    let msg = format!(
                        "Successfully evolved core instructions to version {}. Reason: {}",
                        version, params.reason
                    );

                    // Publish to Redis
                    if let Ok(redis_url) = std::env::var("REDIS_URL") {
                        if let Ok(client) = redis::Client::open(redis_url) {
                            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                                use redis::AsyncCommands;
                                let payload = serde_json::json!({
                                    "conversation_id": conversation_id,
                                    "type": "evolution",
                                    "version": version,
                                    "reason": params.reason
                                })
                                .to_string();
                                let _ = conn
                                    .publish::<&str, String, ()>("nomi:internal_update", payload)
                                    .await;
                            }
                        }
                    }

                    // Dispatch internal update
                    let _ = dispatcher
                        .app_state
                        .dispatch(crate::services::event_dispatcher::AppEvent::broadcast(
                            "evolution",
                            serde_json::json!({
                                "conversation_id": conversation_id,
                                "message": "Nomi has updated her core instructions to better suit your needs. ✨",
                                "reason": params.reason
                            }),
                        ))
                        .await;

                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: msg,
                        follow_up_prompt: "".to_string(),
                        ref_id: version.to_string(),
                    })
                }
                Ok(None) => Ok(ToolResult {
                    error: format!("Error: Conversation ID {} not found.", conversation_id),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
                Err(e) => Ok(ToolResult {
                    error: format!("Database error evolving bootstrap: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
            }
        }
        .boxed()
    }
}
