use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::AnalyzeMediaParameters;
use crate::common::tools::ToolDispatcher;
use base64::Engine;
use dotenvy::var;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;

pub struct AnalyzeMediaPlugin;

impl NomiToolPlugin for AnalyzeMediaPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "analyze_media",
                "Analyze a media file (image, video, audio, or document) and provide information based on a prompt. Use this when the user asks questions about a file, wants to read text from it, or needs a description/summary. If no media_url is provided, it will use the most recently uploaded file in the conversation.",
                None,
            )
            .with_parameters::<AnalyzeMediaParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["ANALYZE_MEDIA", "VISUAL_ANALYSIS", "DOCUMENT_READING", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: AnalyzeMediaParameters = serde_json::from_value(args)?;
            let conversation_id = match dispatcher.conversation_id {
                Some(id) => id,
                None => {
                    return Ok("Conversation ID not found in context".to_string());
                }
            };

            let media_url = if let Some(url) = params.media_url {
                url
            } else {
                // Retrieve from messages table
                match crate::common::repository::message_repo::get_latest_unprocessed_media(
                    &dispatcher.pool,
                    conversation_id,
                )
                .await
                {
                    Ok(Some((url, _type))) => url,
                    Ok(None) => {
                        return Ok("No recent image found to analyze. Please upload an image first!".to_string());
                    }
                    Err(e) => {
                        return Ok(format!("Database error: {}", e));
                    }
                }
            };

            let base_url = var("PUBLIC_GATEWAY_URL").unwrap_or("http://localhost:8000/api".to_string());

            let image_url = if media_url.starts_with("http") && media_url.starts_with(base_url.as_str())
            {
                media_url.replace(format!("{}/files/", base_url).as_str(), "")
            } else {
                media_url.to_string()
            };

            if image_url.starts_with("http") {
                return Ok("Tool doesn't support url from outside app".to_string());
            }
            info!(
                "Analyzing image: {} with prompt: {}",
                image_url, params.prompt
            );

            let data = match dispatcher
                .storage
                .get_file("conversations".to_string(), image_url.clone())
                .await
            {
                Ok(d) => d,
                Err(e) => {
                    return Ok(format!("Storage error: {}", e));
                }
            };

            let mime_type = mime_guess::from_path(&image_url)
                .first_or_octet_stream()
                .to_string();

            let base64_data = base64::engine::general_purpose::STANDARD.encode(data.to_vec());

            let res = match dispatcher
                .gemini
                .generate_content()
                .with_message(gemini_rust::Message {
                    role: gemini_rust::Role::User,
                    content: gemini_rust::Content {
                        parts: Some(vec![
                            gemini_rust::Part::Text {
                                text: params.prompt.clone(),
                                thought: None,
                                thought_signature: None,
                            },
                            gemini_rust::Part::InlineData {
                                inline_data: gemini_rust::Blob {
                                    mime_type,
                                    data: base64_data,
                                },
                                media_resolution: None,
                            },
                        ]),
                        role: Some(gemini_rust::Role::User),
                    },
                })
                .execute()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    return Ok(format!("Gemini error: {}", e));
                }
            };

            Ok(res.text())
        }
        .boxed()
    }
}
