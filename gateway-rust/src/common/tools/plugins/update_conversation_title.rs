use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, UpdateConversationTitleParameters};
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;

pub struct UpdateConversationTitlePlugin;

impl NomiToolPlugin for UpdateConversationTitlePlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "update_conversation_title",
                "Updates the display title or topic name of the current conversation thread or group context inside the database dynamically.",
                None,
            )
            .with_parameters::<UpdateConversationTitleParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["UPDATE_CONVERSATION_TITLE", "RENAME_CHAT", "CHANGE_ROOM_NAME", "DASHBOARD", "COMMUNICATION", "GENERAL"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: UpdateConversationTitleParameters = serde_json::from_value(args)?;
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok(ToolResult {
                        error: "No active conversation context found.".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let result = sqlx::query!(
                "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2",
                params.new_title,
                conversation_id
            )
            .execute(&dispatcher.pool)
            .await;

            if result.is_ok() {
                crate::common::repository::conversation_repo::invalidate_conversation_cache(
                    &dispatcher.app_state.redis,
                    conversation_id
                ).await;
            }

            match result {
                Ok(_) => {
                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: format!(
                            "Successfully changed workspace topic heading to '{}'",
                            params.new_title
                        ),
                        follow_up_prompt: "".to_string(),
                        ref_id: conversation_id.to_string(),
                    })
                }
                Err(e) => Ok(ToolResult {
                    error: format!("Database error updating conversation title: {}", e),
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
