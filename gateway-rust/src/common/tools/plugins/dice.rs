use crate::common::tools::ToolDispatcher;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ToolResult;
use futures::future::{BoxFuture, FutureExt};
use rand::RngExt;
use serde_json::{Value, json};

pub struct DicePlugin;

impl NomiToolPlugin for DicePlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "roll_dice",
            "description": "Roll one or more dice (e.g., 2d6) and return the results.",
            "parameters": {
                "type": "object",
                "properties": {
                    "count": {
                        "type": "integer",
                        "description": "Number of dice to roll (default: 1)",
                        "minimum": 1,
                        "maximum": 10
                    },
                    "sides": {
                        "type": "integer",
                        "description": "Number of sides per die (default: 6)",
                        "minimum": 2,
                        "maximum": 100
                    }
                }
            }
        })
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["GENERAL", "GAMES"]
    }

    fn execute<'a>(
        &'a self,
        _dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let count = args["count"].as_u64().unwrap_or(1) as usize;
            let sides = args["sides"].as_u64().unwrap_or(6);

            let mut rng = rand::rng();
            let rolls: Vec<u64> = (0..count).map(|_| rng.random_range(1..=sides)).collect();
            let total: u64 = rolls.iter().sum();

            let content = if count == 1 {
                format!("🎲 Rolled a d{}: **{}**", sides, total)
            } else {
                format!(
                    "🎲 Rolled {}d{}: **{}** (Details: [{}])",
                    count,
                    sides,
                    total,
                    rolls
                        .iter()
                        .map(|r| r.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };

            Ok(ToolResult {
                error: "".to_string(),
                success: true,
                content,
                follow_up_prompt: "".to_string(),
                ref_id: "".to_string(),
            })
        }
        .boxed()
    }
}
