use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, GetLatestMediaContextParameters};
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;

pub struct GetLatestMediaContextPlugin;

impl NomiToolPlugin for GetLatestMediaContextPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "get_latest_media_context",
                "Retrieves the most recently uploaded file in the conversation context. Use this if the user refers to an image or document without providing a URL.",
                None,
            )
            .with_parameters::<GetLatestMediaContextParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["GET_LATEST_MEDIA", "VISUAL_CONTEXT", "MEDIA_INQUIRY", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        _args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok(ToolResult {
                        error: "No active conversation context".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            match crate::common::repository::message_repo::get_latest_unprocessed_media(
                &dispatcher.pool,
                conversation_id,
            )
            .await
            {
                Ok(Some((_media_url, media_type))) => {
                    let content = format!(
                        "I've retrieved the latest media from our 'Visual Buffer':\n\n- **Type:** {}\n- **Status:** Pending Analysis 🔍\n\nWhat would you like me to do with this? I can log it as an expense, turn it into a sticker, or analyze its content for you! ✨",
                        media_type.to_uppercase(),
                    );

                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content,
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
                Ok(None) => Ok(ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: "Our 'Visual Buffer' is currently empty. No silent media has been captured recently! 🏔️".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
                Err(e) => Ok(ToolResult {
                    error: format!("Database error retrieving media: {}", e),
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
