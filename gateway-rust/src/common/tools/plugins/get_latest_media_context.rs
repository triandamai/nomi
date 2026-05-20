use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::GetLatestMediaContextParameters;
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
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok("No active conversation context".to_string());
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
                        "I've retrieved the latest media from our 'Visual Buffer':


                        - **Type:** {}

                        - **Status:** Pending Analysis 🔍


                        What would you like me to do with this? I can log it as an expense, turn it into a sticker, or analyze its content for you! ✨",
                        media_type.to_uppercase(),
                    );

                    Ok(content)
                }
                Ok(None) => Ok("Our 'Visual Buffer' is currently empty. No silent media has been captured recently! 🏔️".to_string()),
                Err(e) => Ok(format!("Database error retrieving media: {}", e)),
            }
        }
        .boxed()
    }
}
