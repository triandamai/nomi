pub mod tools_model;

use crate::common::tools::tools_model::{
    ExecuteReadQueryParameters, ExecuteReadQueryResponse, ParseToJsonParameters,
    ReadWorkSpaceParameters, ReadWorkSpaceResponse, SearchWebParameters, SearchWebResponse,
    ToolResult, UpdateConversationSoulParameters, UpdateConversationSoulResponse,
};
use gemini_rust::{FunctionDeclaration, Tool};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{Column, Pool, Postgres, Row};
use std::fs;
use std::path::PathBuf;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "tool", content = "args")]
pub enum ArtaTool {
    #[serde(rename = "read_workspace_file")]
    ReadWorkspaceFile {
        params: ReadWorkSpaceParameters,
        user_message: String,
    },
    #[serde(rename = "execute_sql_query")]
    ExecuteSqlQuery {
        params: ExecuteReadQueryParameters,
        user_message: String,
    },
    #[serde(rename = "web_search")]
    WebSearch {
        params: SearchWebParameters,
        user_message: String,
    },
    #[serde(rename = "parse_to_json")]
    ParseStringToJson {
        params: ParseToJsonParameters,
        user_message: String,
    },
    #[serde(rename = "update_nomi_soul")]
    UpdateConversationSoul {
        params: UpdateConversationSoulParameters,
        user_message: String,
    },
}

#[derive(Clone)]
pub struct ToolDispatcher {
    pool: Pool<Postgres>,
    workspace_root: PathBuf,
    conversation_id: Option<Uuid>,
}

impl ToolDispatcher {
    pub fn new(
        pool: Pool<Postgres>,
        workspace_root: PathBuf,
        conversation_id: Option<Uuid>,
    ) -> Self {
        Self {
            pool,
            workspace_root,
            conversation_id,
        }
    }

    pub async fn dispatch(&self, tool: ArtaTool) -> ToolResult {
        match tool {
            ArtaTool::ReadWorkspaceFile {
                params,
                user_message,
            } => self.read_workspace_file(params.path, user_message).await,
            ArtaTool::ExecuteSqlQuery {
                params,
                user_message,
            } => self.execute_sql_query(params.query, user_message).await,
            ArtaTool::WebSearch {
                params,
                user_message,
            } => self.web_search(params.query, user_message).await,
            ArtaTool::ParseStringToJson {
                params,
                user_message,
            } => ToolResult {
                error: "".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
            ArtaTool::UpdateConversationSoul {
                params,
                user_message,
            } => self.update_nomi_soul(params, user_message).await,
        }
    }

    pub fn generate_tool_for_prompt() -> Tool {
        let read_workspace_file = FunctionDeclaration::new(
            "read_workspace_file",
            "Read content of file in workspace",
            None,
        )
        .with_parameters::<ReadWorkSpaceParameters>()
        .with_response::<ReadWorkSpaceResponse>();

        let execute_read_query =
            FunctionDeclaration::new("execute_read_query", "Execute Read Only SQL Query", None)
                .with_parameters::<ExecuteReadQueryParameters>()
                .with_response::<ExecuteReadQueryResponse>();

        let web_search =
            FunctionDeclaration::new("web_search", "Search information from internet", None)
                .with_parameters::<SearchWebParameters>()
                .with_response::<SearchWebResponse>();

        let update_nomi_soul =
            FunctionDeclaration::new(
                "update_nomi_soul",
                "Update Nomi's conversation soul for this session. Provide the new soul content and a witty or logical reason for the evolution.",
                None,
            )
                .with_parameters::<UpdateConversationSoulParameters>()
                .with_response::<UpdateConversationSoulResponse>();

        Tool::with_functions(vec![
            read_workspace_file,
            execute_read_query,
            web_search,
            update_nomi_soul,
        ])
    }

    async fn read_workspace_file(&self, path: String, user_message: String) -> ToolResult {
        debug!(path = %path, "Executing read_workspace_file");

        let requested_path = PathBuf::from(&path);

        if requested_path.is_absolute() || path.contains("..") {
            let msg = "Error: Access denied. Only relative paths within the workspace are allowed."
                .to_string();
            return ToolResult {
                error: msg.clone(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    msg,
                    "read_workspace_file".to_string(),
                ),
            };
        }

        let full_path = self.workspace_root.join(requested_path);

        match fs::read_to_string(full_path) {
            Ok(result) => ToolResult {
                error: "".to_string(),
                success: true,
                content: result.clone(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    result,
                    "read_workspace_file".to_string(),
                ),
            },
            Err(error) => ToolResult {
                error: format!("Error reading file: {}", error),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    error.to_string(),
                    "read_workspace_file".to_string(),
                ),
            },
        }
    }

    async fn execute_sql_query(&self, query: String, user_message: String) -> ToolResult {
        debug!(query = %query, "Executing execute_sql_query");

        let trimmed_query = query.trim().to_uppercase();
        if !trimmed_query.starts_with("SELECT") {
            let msg = "Error: Invalid query format.".to_string();
            return ToolResult {
                error: msg.clone(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    msg,
                    "execute_sql_query".to_string(),
                ),
            };
        }

        match sqlx::query(&query).fetch_all(&self.pool).await {
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

                let json_string =
                    serde_json::to_string_pretty(&json_rows).unwrap_or_else(|_| "[]".to_string());

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: json_string.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        json_string,
                        "execute_sql_query".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("SQL Error: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    e.to_string(),
                    "execute_sql_query".to_string(),
                ),
            },
        }
    }

    async fn web_search(&self, query: String, user_message: String) -> ToolResult {
        debug!(query = %query, "Executing web_search (Mock)");
        ToolResult {
            error: "".to_string(),
            success: true,
            content: format!(
                "Search results for '{}':
1. Result A - info about {}
2. Result B - more context on {}",
                query, query, query
            ),
            follow_up_prompt: build_follow_up_prompt(
                "".to_string(),
                "".to_string(),
                "web_search".to_string(),
            ),
        }
    }
    async fn update_nomi_soul(
        &self,
        params: UpdateConversationSoulParameters,
        user_message: String,
    ) -> ToolResult {
        debug!(
            new_soul = %params.new_soul,
            reason_for_change = %params.reason_for_change,
            "Executing update_nomi_soul"
        );

        let Some(conversation_id) = self.conversation_id else {
            let msg = "Error: No active conversation context for soul update.".to_string();
            return ToolResult {
                error: msg.clone(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    msg,
                    "update_nomi_soul".to_string(),
                ),
            };
        };

        let result: Result<Option<i32>, sqlx::Error> = async {
            let mut tx = self.pool.begin().await?;

            let conversation = sqlx::query("SELECT id FROM conversations WHERE id = $1 FOR UPDATE")
                .bind(conversation_id)
                .fetch_optional(&mut *tx)
                .await?;

            if conversation.is_none() {
                tx.rollback().await?;
                return Ok(None);
            }

            let next_version: i32 = sqlx::query_scalar(
                "SELECT (COALESCE(MAX(version_number), 0) + 1)::INT4 FROM soul_history WHERE conversation_id = $1",
            )
            .bind(conversation_id)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query("UPDATE conversations SET soul_content = $1, updated_at = NOW() WHERE id = $2")
                .bind(&params.new_soul)
                .bind(conversation_id)
                .execute(&mut *tx)
                .await?;

            sqlx::query(
                "INSERT INTO soul_history (conversation_id, soul_content, change_reason, version_number) VALUES ($1, $2, $3, $4)",
            )
            .bind(conversation_id)
            .bind(&params.new_soul)
            .bind(&params.reason_for_change)
            .bind(next_version)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            Ok(Some(next_version))
        }
        .await;

        match result {
            Ok(Some(version)) => {
                let msg = format!(
                    "Successfully updated personality/soul to version {}. Reason: {}",
                    version, params.reason_for_change
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: msg.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "update_nomi_soul".to_string(),
                    ),
                }
            }
            Ok(None) => {
                let msg = format!("Error: Conversation ID {} not found.", conversation_id);
                ToolResult {
                    error: msg.clone(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "update_nomi_soul".to_string(),
                    ),
                }
            }
            Err(e) => {
                let msg = format!("Database error updating conversation soul: {}", e);
                ToolResult {
                    error: msg.clone(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "update_nomi_soul".to_string(),
                    ),
                }
            }
        }
    }
}

fn build_follow_up_prompt(
    user_message: String,
    result_message: String,
    tool_name: String,
) -> String {
    format!(
        "The result for the tool {} '{}' is in. \n
         User's original intent: '{}'. \n\
         Based on the tool output, provide a concise summary or the requested data. \n
         Do not just think; speak to User now.",
        tool_name, user_message, result_message
    )
}
