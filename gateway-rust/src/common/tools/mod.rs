pub mod tools_model;

use crate::Arc;
use crate::common::tools::tools_model::{
    EvolveBootstrapParameters, EvolveBootstrapResponse, ExecuteReadQueryParameters,
    ExecuteReadQueryResponse, ParseToJsonParameters, ReadWebPageParameters, ReadWebPageResponse,
    ReadWorkSpaceParameters, ReadWorkSpaceResponse, SearchWebParameters, SearchWebResponse,
    ToolResult, UpdateConversationSoulParameters, UpdateConversationSoulResponse,
    UpdateKnowledgeBaseParameters, UpdateKnowledgeBaseResponse,
};
use gemini_rust::{FunctionDeclaration, Tool};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{Column, Pool, Postgres, Row};
use std::fs;
use std::path::PathBuf;
use tracing::info;
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
    #[serde(rename = "read_web_page")]
    ReadWebPage {
        params: ReadWebPageParameters,
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
    #[serde(rename = "update_knowledge_base")]
    UpdateKnowledgeBase {
        params: UpdateKnowledgeBaseParameters,
        user_message: String,
    },
    #[serde(rename = "evolve_bootstrap_content")]
    EvolveBootstrap {
        params: EvolveBootstrapParameters,
        user_message: String,
    },
}

#[derive(Clone)]
pub struct ToolDispatcher {
    pool: Pool<Postgres>,
    workspace_root: PathBuf,
    conversation_id: Option<Uuid>,
    gemini: Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    sse: Arc<crate::common::sse::sse_emitter::SseBroadcaster>,
}

impl ToolDispatcher {
    pub fn new(
        pool: Pool<Postgres>,
        workspace_root: PathBuf,
        conversation_id: Option<Uuid>,
        gemini: Arc<gemini_rust::Gemini>,
        gemini_api_key: String,
        sse: Arc<crate::common::sse::sse_emitter::SseBroadcaster>,
    ) -> Self {
        Self {
            pool,
            workspace_root,
            conversation_id,
            gemini,
            gemini_api_key,
            sse,
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
            ArtaTool::ReadWebPage {
                params,
                user_message,
            } => self.read_web_page(params.url, user_message).await,
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
            ArtaTool::UpdateKnowledgeBase {
                params,
                user_message,
            } => self.update_knowledge_base(params, user_message).await,
            ArtaTool::EvolveBootstrap {
                params,
                user_message,
            } => self.evolve_bootstrap(params, user_message).await,
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

        let read_web_page = FunctionDeclaration::new(
            "read_web_page",
            "Read content of a web page as Markdown. Best for technical docs or news.",
            None,
        )
        .with_parameters::<ReadWebPageParameters>()
        .with_response::<ReadWebPageResponse>();

        let update_nomi_soul =
            FunctionDeclaration::new(
                "update_nomi_soul",
                "Update Nomi's conversation soul for this session. Provide the new soul content and a witty or logical reason for the evolution.",
                None,
            )
                .with_parameters::<UpdateConversationSoulParameters>()
                .with_response::<UpdateConversationSoulResponse>();

        let update_knowledge_base =
            FunctionDeclaration::new(
                "update_knowledge_base",
                "Save specific facts, preferences, and project details immediately to long-term memory. This updates your permanent knowledge base.",
                None,
            )
                .with_parameters::<UpdateKnowledgeBaseParameters>()
                .with_response::<UpdateKnowledgeBaseResponse>();

        let evolve_bootstrap_content = FunctionDeclaration::new(
            "evolve_bootstrap_content",
            "Update your own personality or mission instructions (System Prompt) dynamically.",
            None,
        )
        .with_parameters::<EvolveBootstrapParameters>()
        .with_response::<EvolveBootstrapResponse>();

        Tool::with_functions(vec![
            read_workspace_file,
            execute_read_query,
            web_search,
            read_web_page,
            update_nomi_soul,
            update_knowledge_base,
            evolve_bootstrap_content,
        ])
    }

    async fn read_workspace_file(&self, path: String, user_message: String) -> ToolResult {
        info!(path = %path, "Executing read_workspace_file");

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
        info!(query = %query, "Executing execute_sql_query");

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
        info!(query = %query, "Executing web_search");

        let api_key = match std::env::var("TAVILY_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return ToolResult {
                    error: "TAVILY_API_KEY not found in environment".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
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
                    output.push_str(&format!("Summary: {}\n\n", answer));
                }

                if let Some(results) = results {
                    for (i, res) in results.iter().enumerate() {
                        let title = res["title"].as_str().unwrap_or("No Title");
                        let url = res["url"].as_str().unwrap_or("No URL");
                        let content = res["content"].as_str().unwrap_or("");
                        output.push_str(&format!(
                            "{}. {} \nURL: {} \nSnippet: {}\n\n",
                            i + 1,
                            title,
                            url,
                            content
                        ));
                    }
                }

                info!("get result from web search and returning to agent");
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: output.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        output,
                        "web_search".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Tavily API error: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn read_web_page(&self, url: String, user_message: String) -> ToolResult {
        info!(url = %url, "Executing read_web_page via Jina Reader");

        let client = reqwest::Client::new();
        let jina_url = format!("https://r.jina.ai/{}", url);

        let api_key = match std::env::var("JINA_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return ToolResult {
                    error: "JINA_API_KEY not found in environment".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let res = client
            .get(jina_url)
            .header("X-Return-Format", "markdown")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await;

        match res {
            Ok(response) => {
                let mut content = response.text().await.unwrap_or_default();

                // Safety & Token Budget: Limit to roughly 4000 tokens (~16000 chars)
                if content.len() > 16000 {
                    content.truncate(16000);
                    content.push_str("\n\n[Content truncated for token budget...]");
                }

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        "Web content retrieved successfully.".to_string(),
                        "read_web_page".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Jina Reader error: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn update_nomi_soul(
        &self,
        params: UpdateConversationSoulParameters,
        user_message: String,
    ) -> ToolResult {
        info!(
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

            let convo = sqlx::query!(
                "SELECT soul_content, bootstrap_content FROM conversations WHERE id = $1 FOR UPDATE",
                conversation_id
            )
            .fetch_one(&mut *tx)
            .await?;

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
                "INSERT INTO soul_history (conversation_id, soul_content, bootstrap, change_reason, version_number) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(conversation_id)
            .bind(&params.new_soul)
            .bind(convo.bootstrap_content)
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
    async fn evolve_bootstrap(
        &self,
        params: EvolveBootstrapParameters,
        user_message: String,
    ) -> ToolResult {
        let conversation_id = match self.conversation_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "No active conversation to evolve.".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let result: Result<Option<i32>, sqlx::Error> = async {
            let mut tx = self.pool.begin().await?;

            let convo = sqlx::query!(
                "SELECT soul_content, bootstrap_content FROM conversations WHERE id = $1 FOR UPDATE",
                conversation_id
            )
            .fetch_one(&mut *tx)
            .await?;

            let next_version: i32 = sqlx::query_scalar(
                "SELECT (COALESCE(MAX(version_number), 0) + 1)::INT4 FROM soul_history WHERE conversation_id = $1",
            )
            .bind(conversation_id)
            .fetch_one(&mut *tx)
            .await?;

            sqlx::query(
                "UPDATE conversations SET bootstrap_content = $1, updated_at = NOW() WHERE id = $2",
            )
            .bind(&params.updated_instructions)
            .bind(conversation_id)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO soul_history (conversation_id, soul_content, bootstrap, change_reason, version_number) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(conversation_id)
            .bind(convo.soul_content)
            .bind(&params.updated_instructions)
            .bind(&params.reason)
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
                    "Successfully evolved core instructions to version {}. Reason: {}",
                    version, params.reason
                );

                // Publish to Redis
                if let Ok(redis_url) = std::env::var("REDIS_URL") {
                    if let Ok(client) = redis::Client::open(redis_url) {
                        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                            use redis::AsyncCommands;
                            let payload = serde_json::json!({
                                "conversation_id": conversation_id,
                                "type": "evolution",
                                "version": version,
                                "reason": params.reason
                            })
                            .to_string();
                            let _ = conn
                                .publish::<&str, String, ()>("nomi:internal_update", payload)
                                .await;
                        }
                    }
                }

                // Broadcast SSE
                let _ = self
                    .sse
                    .send(crate::common::sse::sse_builder::SseBuilder::new(
                        crate::common::sse::sse_builder::SseTarget::broadcast(
                            "evolution".to_string(),
                        ),
                        serde_json::json!({
                            "conversation_id": conversation_id,
                            "message": "Nomi has updated her core instructions to better suit your needs. ✨",
                            "reason": params.reason
                        }),
                    ))
                    .await;

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: msg.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "evolve_bootstrap_content".to_string(),
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
                        "evolve_bootstrap_content".to_string(),
                    ),
                }
            }
            Err(e) => {
                let msg = format!("Database error evolving bootstrap: {}", e);
                ToolResult {
                    error: msg.clone(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "evolve_bootstrap_content".to_string(),
                    ),
                }
            }
        }
    }

    async fn update_knowledge_base(
        &self,
        params: UpdateKnowledgeBaseParameters,
        user_message: String,
    ) -> ToolResult {
        let summarizer_prompt = format!(
            "Analyze the following content and return a JSON object with:
1. 'summary': A concise summary of the facts or details.
2. 'nodes': An array of entities ({{'id': 'unique_id', 'label': 'Entity Name', 'node_type': 'Technology|Project|Person|Organization|Memory'}}).
3. 'edges': An array of relationships ({{'source': 'node_id', 'target': 'node_id', 'relationship': 'Description'}}).

Rules:
- 'id' should be lowercase and snake_case.
- Focus on the core 'Atomic Truth' being saved.

Content:
{}
",
            params.content
        );

        let summary_res = self
            .gemini
            .generate_content()
            .with_user_message(summarizer_prompt)
            .execute()
            .await;

        let parsed_data = match summary_res {
            Ok(resp) => {
                let raw_json = resp.text();
                if let Some(start) = raw_json.find('{') {
                    if let Some(end) = raw_json.rfind('}') {
                        serde_json::from_str(&raw_json[start..=end]).unwrap_or(serde_json::json!({
                            "summary": params.content,
                            "nodes": [],
                            "edges": []
                        }))
                    } else {
                        serde_json::json!({"summary": params.content, "nodes": [], "edges": []})
                    }
                } else {
                    serde_json::json!({"summary": params.content, "nodes": [], "edges": []})
                }
            }
            Err(_) => {
                serde_json::json!({"summary": params.content, "nodes": [], "edges": []})
            }
        };

        let summary_text = parsed_data["summary"]
            .as_str()
            .unwrap_or(&params.content)
            .to_string();

        if let Ok(embedding) = crate::rag::get_embedding(&self.gemini_api_key, &summary_text).await
        {
            let metadata = serde_json::json!({
                "type": "memory",
                "category": params.category,
                "graph": {
                    "nodes": parsed_data["nodes"],
                    "links": parsed_data["edges"]
                }
            });

            let save_result = crate::rag::save_to_knowledge_base(
                &self.pool,
                &summary_text,
                embedding,
                Some(metadata),
            )
            .await;

            match save_result {
                Ok(_) => {
                    let msg = format!("Successfully saved to knowledge base: {}", params.category);
                    ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: msg.clone(),
                        follow_up_prompt: build_follow_up_prompt(
                            user_message,
                            msg,
                            "update_knowledge_base".to_string(),
                        ),
                    }
                }
                Err(e) => {
                    let msg = format!("Error saving to knowledge base: {}", e);
                    ToolResult {
                        error: msg.clone(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: build_follow_up_prompt(
                            user_message,
                            msg,
                            "update_knowledge_base".to_string(),
                        ),
                    }
                }
            }
        } else {
            let msg = "Error generating embedding for knowledge base update.".to_string();
            ToolResult {
                error: msg.clone(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: build_follow_up_prompt(
                    user_message,
                    msg,
                    "update_knowledge_base".to_string(),
                ),
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
