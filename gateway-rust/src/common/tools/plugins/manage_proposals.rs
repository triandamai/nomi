use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use anyhow::Result;
use serde_json::{json, Value};
use futures::future::{BoxFuture, FutureExt};

pub struct ManageSkillProposalsPlugin;

impl NomiToolPlugin for ManageSkillProposalsPlugin {
    fn matching_intents(&self) -> &[&str] {
        &["MANAGE_SKILL_PROPOSALS", "SYSTEM_INTERNAL_DISCOVERY"]
    }

    fn rules(&self) -> &str {
        "Use this tool to check if a skill proposal already exists or to update the user on the status of their requested features."
    }

    fn schema(&self) -> Value {
        json!({
            "name": "manage_skill_proposals",
            "description": "List or search for existing skill blueprints in the Agent Factory. Use this to check if a feature is already being built or to check the status of a proposal (pending, processing, ready, failed, deployed).",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": { 
                        "type": "string", 
                        "enum": ["list", "search"],
                        "description": "The action to perform. 'list' returns recent proposals, 'search' filters by slug or name."
                    },
                    "query": { 
                        "type": "string", 
                        "description": "Optional search term for the 'search' action."
                    }
                },
                "required": ["action"]
            }
        })
    }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, Result<String>> {
        async move {
            let action = args["action"].as_str().unwrap_or("list");
            let query = args["query"].as_str().unwrap_or("");

            let res = if action == "search" && !query.is_empty() {
                sqlx::query(
                    "SELECT slug, name, status, created_at FROM plugin_creation_suggestions \
                     WHERE slug ILIKE $1 OR name ILIKE $1 \
                     ORDER BY created_at DESC LIMIT 10"
                )
                .bind(format!("%{}%", query))
                .fetch_all(&dispatcher.pool)
                .await?
            } else {
                sqlx::query(
                    "SELECT slug, name, status, created_at FROM plugin_creation_suggestions \
                     ORDER BY created_at DESC LIMIT 10"
                )
                .fetch_all(&dispatcher.pool)
                .await?
            };

            if res.is_empty() {
                return Ok("No matching skill proposals found in the factory queue.".to_string());
            }

            use sqlx::Row;
            let mut output = String::from("### Agent Factory Staging Queue:\n");
            for row in res {
                let slug: String = row.get("slug");
                let name: String = row.get("name");
                let status: String = row.get("status");
                let created: chrono::DateTime<chrono::Utc> = row.get("created_at");
                
                output.push_str(&format!(
                    "- **{}** (slug: `{}`) | Status: **{}** | Proposed: {}\n",
                    name, slug, status.to_uppercase(), created.format("%Y-%m-%d %H:%M")
                ));
            }

            Ok(output)
        }
        .boxed()
    }
}
