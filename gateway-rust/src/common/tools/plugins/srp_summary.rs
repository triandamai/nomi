use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use anyhow::Result;
use serde_json::{json, Value};
use futures::future::{BoxFuture, FutureExt};

pub struct SrpSummaryPlugin;

impl NomiToolPlugin for SrpSummaryPlugin {
    fn matching_intents(&self) -> &[&str] {
        &["GENERAL", "SYSTEM_INTERNAL_DISCOVERY"]
    }

    fn rules(&self) -> &str {
        "Use this tool to provide the user with a summary of how you have autonomously evolved your tool-handling logic (learned phrases, new rules)."
    }

    fn schema(&self) -> Value {
        json!({
            "name": "get_srp_summary",
            "description": "Retrieve a summary of Nomi's self-reinforcement status. Lists which tools have been optimized, how many new rules have been learned, and common vocabulary expansions.",
            "parameters": {
                "type": "object",
                "properties": {
                    "plugin_slug": { 
                        "type": "string", 
                        "description": "Optional: Filter summary for a specific tool handle (e.g., 'manage_finance')."
                    }
                }
            }
        })
    }

    fn execute<'a>(&'a self, dispatcher: &'a ToolDispatcher, args: Value) -> BoxFuture<'a, Result<String>> {
        async move {
            let plugin_slug = args["plugin_slug"].as_str();

            let res = if let Some(slug) = plugin_slug {
                sqlx::query(
                    "SELECT plugin_slug, enriched_description, additional_rules, learned_phrases, updated_at \
                     FROM static_plugin_reinforcements WHERE plugin_slug = $1"
                )
                .bind(slug)
                .fetch_all(&dispatcher.pool)
                .await?
            } else {
                sqlx::query(
                    "SELECT plugin_slug, enriched_description, additional_rules, learned_phrases, updated_at \
                     FROM static_plugin_reinforcements ORDER BY updated_at DESC LIMIT 10"
                )
                .fetch_all(&dispatcher.pool)
                .await?
            };

            if res.is_empty() {
                return Ok("I haven't performed any autonomous reinforcement passes yet. My core logic is currently running on standard static definitions.".to_string());
            }

            use sqlx::Row;
            let mut output = String::from("### 🧠 My Self-Reinforcement Audit:\n");
            output.push_str("I have been autonomously optimizing my core logic based on our interactions. Here is a summary of my evolution:\n\n");

            for row in res {
                let slug: String = row.get("plugin_slug");
                let rules: Vec<String> = row.get("additional_rules");
                let phrases: Vec<String> = row.get("learned_phrases");
                let updated: chrono::DateTime<chrono::Utc> = row.get("updated_at");
                
                output.push_str(&format!("#### Tool: `{}`\n", slug));
                output.push_str(&format!("- **Learned Rules:** {} active guardrails.\n", rules.len()));
                output.push_str(&format!("- **Vocabulary:** {} custom phrases/keywords cataloged.\n", phrases.len()));
                output.push_str(&format!("- **Last Evolved:** {}\n\n", updated.format("%Y-%m-%d %H:%M")));
            }

            output.push_str("---\n*I am constantly refining these rules in the background to better align with your specific workflow. ✨*");

            Ok(output)
        }
        .boxed()
    }
}
