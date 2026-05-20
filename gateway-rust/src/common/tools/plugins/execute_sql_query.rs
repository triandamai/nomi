use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::ExecuteReadQueryParameters;
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::{json, Map, Value};
use sqlx::{Column, Row};
use tracing::info;

pub struct ExecuteSqlQueryPlugin;

impl NomiToolPlugin for ExecuteSqlQueryPlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new("execute_read_query", "Execute Read Only SQL Query", None)
                .with_parameters::<ExecuteReadQueryParameters>(),
        )
        .unwrap()
    }

    fn rules(&self) -> &str {
        ""
    }

    fn matching_intents(&self) -> &[&str] {
        &["EXECUTE_SQL", "DATABASE_QUERY", "READ_ONLY_SQL", "STORAGE", "FULL_REGISTRY"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: ExecuteReadQueryParameters = serde_json::from_value(args)?;
            info!(query = %params.query, "Executing execute_sql_query via plugin");

            let trimmed_query = params.query.trim().to_uppercase();
            if !trimmed_query.starts_with("SELECT") {
                return Ok("Error: Invalid query format. Only SELECT queries are allowed.".to_string());
            }

            match sqlx::query(&params.query).fetch_all(&dispatcher.pool).await {
                Ok(rows) => {
                    let mut json_rows = Vec::new();

                    for row in rows {
                        let mut map = Map::new();
                        for column in row.columns() {
                            let name = column.name();

                            // Optimization: Skip embedding columns to save tokens
                            if name.contains("embedding") || name.contains("vector") {
                                continue;
                            }

                            // Try to get value as Value directly if supported, or fall back to String then null-strip
                            let value: Value = row
                                .try_get::<String, _>(name)
                                .map(|s| json!(s))
                                .unwrap_or(Value::Null);

                            // Optimization: Strip null values to save tokens
                            if !value.is_null() {
                                map.insert(name.to_string(), value);
                            }
                        }
                        json_rows.push(Value::Object(map));
                    }

                    Ok(serde_json::to_string_pretty(&json_rows).unwrap_or_else(|_| "[]".to_string()))
                }
                Err(e) => Ok(format!("SQL Error: {}", e)),
            }
        }
        .boxed()
    }
}
