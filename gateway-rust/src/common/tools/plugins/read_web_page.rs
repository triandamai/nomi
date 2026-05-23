use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;
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

    fn rules(&self) -> &str {
        "### WEB LOGIC\n- Use `web_search` and `read_web_page` to scour the web.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["GENERAL", "RESEARCH", "NEWS", "DOCUMENTATION", "WEB"]
    }

    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
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
                    return Ok(ToolResult {
                        error: "Failed read web page: JINA_API_KEY not found".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
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
                        content.push_str("\n\n[Content truncated for token budget...]");
                    }

                    if content.trim().is_empty() {
                        return Ok(ToolResult {
                            error: "".to_string(),
                            success: true,
                            content: "I checked the link, but I couldn't find any readable text there! 🏔️".to_string(),
                            follow_up_prompt: "".to_string(),
                            ref_id: "".to_string(),
                        });
                    }

                    Ok(ToolResult {
                        error: "".to_string(),
                        success: true,
                        content,
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
                Err(e) => {
                    info!("Error execute jina: {}", e);
                    Ok(ToolResult {
                        error: format!("Error reading web page: {}", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    })
                }
            }
        }
        .boxed()
    }
}
