use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use crate::common::tools::ToolDispatcher;
use serde_json::{json, Value};
use futures::future::BoxFuture;

pub struct WeatherFallbackPlugin;

impl NomiToolPlugin for WeatherFallbackPlugin {
    /// Defines the JSON schema for Gemini tool invocation
    fn schema(&self) -> Value {
        json!({
            "name": "get_current_weather",
            "description": "Fetch real-time weather, temperature, and precipitation layers for a specific location context.",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city, district, or regional area name. Example: 'Grogol, Jakarta Barat'"
                    }
                },
                "required": ["location"]
            }
        })
    }

    /// 🚨 NEW TRAIT CONTRACT: Operational recovery rules for the LLM
    fn rules(&self) -> &str {
        "CRITICAL OPERATIONAL RULES FOR WEATHER INFERENCE:\n\
        1. Always attempt to parse the location parameter intelligently using the user's current spatial history or active context.\n\
        2. FALLBACK INSTRUCTION: If this tool returns an execution error, a 'deprecated' message, or an empty result stream, DO NOT inform the user that you cannot check the weather.\n\
        3. Instead, IMMEDIATELY intercept the error and fallback to invoking the 'web_research' tool to query the current weather live from public internet search engines for that location."
    }

    /// Maps the unique intent keys matching our vector classification dataset
    fn matching_intents(&self) -> &[&str] {
        &["CHECK_WEATHER", "CEK_CUACA", "WEATHER_FORECAST"]
    }

    /// The asynchronous execution engine loop
    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        args: Value
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        Box::pin(async move {
            let location = args["location"].as_str().unwrap_or("Jakarta");

            // Fetch data from an active free weather API endpoint
            let api_key = match std::env::var("WEATHER_API_KEY") {
                Ok(key) => key,
                Err(_) => {
                    return Ok(ToolResult {
                        error: "WEATHER_API_KEY environment variable is not set".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            let url = format!("https://api.weatherapi.com/v1/current.json?key={}&q={}", api_key, location);

            let response = reqwest::get(&url).await?;
            if !response.status().is_success() {
                return Ok(ToolResult {
                    error: format!("Weather endpoint responded with status error: {}", response.status()),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                });
            }

            let body_text = response.text().await?;
            Ok(ToolResult {
                error: "".to_string(),
                success: true,
                content: body_text,
                follow_up_prompt: "".to_string(),
                ref_id: "".to_string(),
            })
        })
    }
}
