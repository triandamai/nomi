use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use anyhow::Result;
use serde_json::json;
use futures::future::{BoxFuture, FutureExt};
use serde_json::Value;

pub struct SuggestSkillPlugin;

impl NomiToolPlugin for SuggestSkillPlugin {
    fn matching_intents(&self) -> &[&str] {
        &["SKILL","PLUGIN"] // Triggered via conversation when Nomi detects a capability gap
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
                    "schema_json": { "type": "object", "description": "The JSON schema for tool parameters" },
                    "how_it_works": { "type": "string", "description": "Logical roadmap or algorithm for the SWE agent to follow" }
                },
                "required": ["slug", "name", "description", "schema_json", "how_it_works"]
            }
        })
    }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, Result<String>> {
        async move {
            let slug = args["slug"].as_str().unwrap_or_default();
            let name = args["name"].as_str().unwrap_or_default();
            let description = args["description"].as_str().unwrap_or_default();
            let schema_json = args["schema_json"].clone();
            let how_it_works = args["how_it_works"].as_str().unwrap_or_default();

            sqlx::query(
                "INSERT INTO plugin_creation_suggestions (slug, name, description, schema_json, how_it_works, status) \
                 VALUES ($1, $2, $3, $4, $5, 'pending') \
                 ON CONFLICT (slug) DO UPDATE SET \
                    description = EXCLUDED.description, \
                    schema_json = EXCLUDED.schema_json, \
                    how_it_works = EXCLUDED.how_it_works, \
                    updated_at = NOW()"
            )
            .bind(slug)
            .bind(name)
            .bind(description)
            .bind(schema_json)
            .bind(how_it_works)
            .execute(&dispatcher.pool)
            .await?;

            Ok(format!("Success: I have submitted a blueprint for the [{}] skill to the Distributed Agent Factory. You can review and approve the build in the SRP Factory Console.", name))
        }
        .boxed()
    }
}
