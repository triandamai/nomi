use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use tracing::info;

pub struct WebSearchPlugin;

impl NomiToolPlugin for WebSearchPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "web_search",
            "description": "Search information from internet",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query to look up on the internet"
                    },
                    "user_message": {
                        "type": "string",
                        "description": "The original user message to provide context for the search"
                    }
                },
                "required": ["query", "user_message"]
            }
        })
    }

    fn rules(&self) -> &str {
        "### WEB LOGIC\n- Use `web_search` and `read_web_page` to scour the web.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["GENERAL", "RESEARCH", "NEWS", "WEB","RETRIEVE_KNOWLEDGE","SEARCH","KEYWORD"]
    }

    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let query = args["query"].as_str().unwrap_or_default();
            let _user_message = args["user_message"].as_str().unwrap_or_default();

            info!(query = %query, "Executing web_search plugin");

            let api_key = match std::env::var("TAVILY_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    info!("TAVILY_API_KEY not found in environment");
                    return Ok("Cannot reach website search: TAVILY_API_KEY not found".to_string());
                }
            };

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.tavily.com/search")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&json!({
                    "query": query,
                    "search_depth": "advanced",
                    "include_answer": true,
                    "max_results": 5
                }))
                .send()
                .await;

            match res {
                Ok(response) => {
                    let val: Value = response.json().await.unwrap_or(json!({}));
                    let results = val["results"].as_array();

                    let mut output = String::new();
                    if let Some(answer) = val["answer"].as_str() {
                        output.push_str(&format!("Summary: {}

", answer));
                    }

                    if let Some(results) = results {
                        for (i, res) in results.iter().enumerate() {
                            let title = res["title"].as_str().unwrap_or("No Title");
                            let url = res["url"].as_str().unwrap_or("No URL");
                            let content = res["content"].as_str().unwrap_or("");
                            output.push_str(&format!(
                                "{}. {} 
URL: {} 
Snippet: {}

",
                                i + 1,
                                title,
                                url,
                                content
                            ));
                        }
                    }

                    info!("get result from web search and returning to agent");
                    Ok(output)
                }
                Err(e) => {
                    info!("Error execute tavily: {}", e);
                    Ok(format!("Web search API error: {}", e))
                }
            }
        }
        .boxed()
    }
}
