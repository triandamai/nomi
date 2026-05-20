use crate::common::tools::ToolDispatcher;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{Value, json};
use chrono::NaiveDate;
use sqlx::Row;

pub struct HealthPlugin;

impl NomiToolPlugin for HealthPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "manage_health_data",
            "description": "Query or summarize user health and vitality biometrics (steps, sleep, heart rate, workouts).",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["query_daily", "query_range", "get_summary"],
                        "description": "The type of health data operation to perform."
                    },
                    "start_date": {
                        "type": "string",
                        "description": "Start date in YYYY-MM-DD format (ISO 8601)."
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in YYYY-MM-DD format (ISO 8601)."
                    }
                },
                "required": ["action"]
            }
        })
    }

    fn rules(&self) -> &str {
        "### VITALITY LOGIC\n- Use `manage_health_data` to query or summarize biometrics (steps, sleep, heart rate).\n- Be a proactive personal companion. Notice physical strains (low sleep, high heart rates from trekking/motorcycling) and provide tailored insights. 🥗✨\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["VITALITY", "DASHBOARD"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let user_id = match dispatcher.user_id {
                Some(id) => id,
                None => return Ok("Authentication required to access health metrics.".to_string()),
            };

            let start_date = args["start_date"]
                .as_str()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            let end_date = args["end_date"]
                .as_str()
                .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

            let rows = sqlx::query(
                r#"
                SELECT log_date, metrics
                FROM user_health_metrics
                WHERE user_id = $1
                  AND ($2::DATE IS NULL OR log_date >= $2)
                  AND ($3::DATE IS NULL OR log_date <= $3)
                ORDER BY log_date ASC
                "#,
            )
            .bind(user_id)
            .bind(start_date)
            .bind(end_date)
            .fetch_all(&dispatcher.pool)
            .await?;

            if rows.is_empty() {
                return Ok("No health metrics found for the specified period.".to_string());
            }

            let mut content = String::new();
            for row in rows {
                let log_date: NaiveDate = row.get("log_date");
                let metrics: Value = row.get("metrics");
                content.push_str(&format!(
                    "Date: {}
Metrics: {}

",
                    log_date,
                    serde_json::to_string(&metrics).unwrap_or_default()
                ));
            }

            Ok(content)
        }
        .boxed()
    }
}
