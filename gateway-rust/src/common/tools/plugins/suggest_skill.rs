use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use crate::common::tools::ToolDispatcher;
use anyhow::Result;
use serde_json::json;
use futures::future::{BoxFuture, FutureExt};
use serde_json::Value;

pub struct SuggestSkillPlugin;

impl NomiToolPlugin for SuggestSkillPlugin {
    fn matching_intents(&self) -> &[&str] {
        &["SUGGEST_NEW_SKILL", "GENERAL"] // Explicit intent for direct triggering
    }

    fn rules(&self) -> &str {
        ""
    }

    fn schema(&self) -> Value {
        json!({
            "name": "suggest_new_skill",
            "description": "Suggest a new autonomous plugin/skill for Nomi to build. Use this when User asks for a feature that doesn't exist yet.",
            "parameters": {
                "type": "object",
                "properties": {
                    "slug": { "type": "string", "description": "Unique snake_case identifier, e.g., 'crypto_tracker'" },
                    "name": { "type": "string", "description": "Human-readable name" },
                    "description": { "type": "string", "description": "What this tool does in detail" },
                    "intents": { 
                        "type": "array", 
                        "items": { "type": "string" },
                        "description": "List of intent keywords that should trigger this tool, e.g., ['CRYPTO_PRICE', 'TRACK_PORTFOLIO']"
                    },
                    "schema_json": { "type": "object", "description": "The JSON schema for tool parameters" },
                    "how_it_works": { "type": "string", "description": "Logical roadmap or algorithm for the SWE agent to follow" }
                },
                "required": ["slug", "name", "description", "intents", "schema_json", "how_it_works"]
            }
        })
    }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, Result<ToolResult>> {
        async move {
            let slug = args["slug"].as_str().unwrap_or_default();
            let name = args["name"].as_str().unwrap_or_default();
            let description = args["description"].as_str().unwrap_or_default();
            let schema_json = args["schema_json"].clone();
            let how_it_works = args["how_it_works"].as_str().unwrap_or_default();
            
            // Extract intents array
            let intents: Vec<String> = args["intents"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            sqlx::query(
                "INSERT INTO plugin_creation_suggestions (slug, name, description, schema_json, how_it_works, intents, status) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'pending') \
                 ON CONFLICT (slug) DO UPDATE SET \
                    description = EXCLUDED.description, \
                    schema_json = EXCLUDED.schema_json, \
                    how_it_works = EXCLUDED.how_it_works, \
                    intents = EXCLUDED.intents, \
                    updated_at = NOW()"
            )
            .bind(slug)
            .bind(name)
            .bind(description)
            .bind(schema_json)
            .bind(how_it_works)
            .bind(&intents)
            .execute(&dispatcher.pool)
            .await?;

            Ok(ToolResult {
                error: "".to_string(),
                success: true,
                content: format!("Success: I have submitted a blueprint for the [{}] skill with {} intents to the Distributed Agent Factory. You can review and approve the build in the Factory Console.", name, intents.len()),
                follow_up_prompt: "".to_string(),
                ref_id: slug.to_string(),
            })
        }
        .boxed()
    }
}
