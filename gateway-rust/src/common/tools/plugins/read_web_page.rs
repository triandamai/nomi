use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use tracing::info;

pub struct ReadWebPagePlugin;

impl NomiToolPlugin for ReadWebPagePlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "read_web_page",
            "description": "Read content of a web page as Markdown. Best for technical docs or news.",
            "parameters": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL of the web page to read"
                    },
                    "user_message": {
                        "type": "string",
                        "description": "The original user message to provide context for reading"
                    }
                },
                "required": ["url", "user_message"]
            }
        })
    }

    fn matching_intents(&self) -> &[&str] {
        &["GENERAL", "RESEARCH", "NEWS", "DOCUMENTATION"]
    }

    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let url = args["url"].as_str().unwrap_or_default();
            let _user_message = args["user_message"].as_str().unwrap_or_default();

            info!(url = %url, "Executing read_web_page plugin via Jina Reader");

            let client = reqwest::Client::new();
            let jina_url = format!("https://r.jina.ai/{}", url);

            let api_key = match std::env::var("JINA_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    info!("JINA_API_KEY not found in environment");
                    return Ok("Failed read web page: JINA_API_KEY not found".to_string());
                }
            };

            let res = client
                .get(jina_url)
                .header("X-Return-Format", "markdown")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await;

            match res {
                Ok(response) => {
                    let mut content = response.text().await.unwrap_or_default();

                    // Safety & Token Budget: Limit to roughly 1250 tokens (~5000 chars)
                    if content.len() > 5000 {
                        content = content.chars().take(5000).collect::<String>();
                        content.push_str("

[Content truncated for token budget...]");
                    }

                    if content.trim().is_empty() {
                        return Ok("I checked the link, but I couldn't find any readable text there! 🏔️".to_string());
                    }

                    Ok(content)
                }
                Err(e) => {
                    info!("Error execute jina: {}", e);
                    Ok(format!("Error reading web page: {}", e))
                }
            }
        }
        .boxed()
    }
}
