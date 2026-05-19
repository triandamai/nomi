use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::UpdateConversationSoulParameters;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;

pub struct UpdateConversationSoulPlugin;

impl NomiToolPlugin for UpdateConversationSoulPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "update_nomi_soul",
                "Update Nomi's conversation soul for this session. Provide the new soul content and a witty or logical reason for the evolution.",
                None,
            )
            .with_parameters::<UpdateConversationSoulParameters>()
        ).unwrap()
    }

    fn matching_intents(&self) -> &[&str] {
        &["UPDATE_CONVERSATION_SOUL", "CHANGE_NOMI_PERSONALITY", "ALTER_AI_BEHAVIOR", "MUTATE_SOUL_METRICS", "DASHBOARD", "GENERAL"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: UpdateConversationSoulParameters = serde_json::from_value(args)?;
            info!(
                new_soul = %params.new_soul,
                reason_for_change = %params.reason_for_change,
                "Executing update_nomi_soul via plugin"
            );

            let Some(conversation_id) = dispatcher.conversation_id else {
                return Ok("Error: No active conversation context for soul update.".to_string());
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

                sqlx::query("UPDATE conversations SET soul_content = $1, updated_at = NOW() WHERE id = $2")
                    .bind(&params.new_soul)
                    .bind(conversation_id)
                    .execute(&mut *tx)
                    .await?;

                sqlx::query(
                    "INSERT INTO soul_history (conversation_id, soul_content, bootstrap, change_reason, version_number) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(conversation_id)
                .bind(&params.new_soul)
                .bind(convo.bootstrap_content)
                .bind(&params.reason_for_change)
                .bind(next_version)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
                Ok(Some(next_version))
            }
            .await;

            match result {
                Ok(Some(version)) => Ok(format!(
                    "Successfully updated personality/soul to version {}. Reason: {}",
                    version, params.reason_for_change
                )),
                Ok(None) => Ok(format!("Error: Conversation ID {} not found.", conversation_id)),
                Err(e) => Ok(format!("Database error updating conversation soul: {}", e)),
            }
        }
        .boxed()
    }
}
