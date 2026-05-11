pub mod tools_model;

use crate::Arc;
use crate::common::tools::tools_model::{
    CreateReminderParameters, CreateReminderResponse, EvolveBootstrapParameters,
    EvolveBootstrapResponse, ExecuteReadQueryParameters, ExecuteReadQueryResponse,
    GetInboxSummaryParameters, GetInboxSummaryResponse, GetLatestMediaContextParameters,
    GetLatestMediaContextResponse, GetReminderStatsParameters, GetReminderStatsResponse,
    MakeStickerParameters, MakeStickerResponse, ModifyReminderParameters, ModifyReminderResponse,
    ParseToJsonParameters, ReadWebPageParameters, ReadWebPageResponse, ReadWorkSpaceParameters,
    ReadWorkSpaceResponse, SearchUsersParameters, SearchUsersResponse, SearchWebParameters,
    SearchWebResponse, SendDirectMessageParameters, SendDirectMessageResponse, ToolResult,
    UpdateConversationSoulParameters, UpdateConversationSoulResponse,
    UpdateKnowledgeBaseParameters, UpdateKnowledgeBaseResponse, UpdateUserProfileParameters,
    UpdateUserProfileResponse,
};
use crate::prompts::PromptRegistry;
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
    #[serde(rename = "create_reminder")]
    CreateReminder {
        params: CreateReminderParameters,
        user_message: String,
    },
    #[serde(rename = "modify_reminder")]
    ModifyReminder {
        params: ModifyReminderParameters,
        user_message: String,
    },
    #[serde(rename = "get_inbox_summary")]
    GetInboxSummary {
        params: GetInboxSummaryParameters,
        user_message: String,
    },
    #[serde(rename = "get_reminder_stats")]
    GetReminderStats {
        params: GetReminderStatsParameters,
        user_message: String,
    },
    #[serde(rename = "search_users")]
    SearchUsers {
        params: SearchUsersParameters,
        user_message: String,
    },
    #[serde(rename = "update_user_profile")]
    UpdateUserProfile {
        params: UpdateUserProfileParameters,
        user_message: String,
    },
    #[serde(rename = "send_direct_message")]
    SendDirectMessage {
        params: SendDirectMessageParameters,
        user_message: String,
    },
    #[serde(rename = "make_sticker")]
    MakeSticker {
        params: MakeStickerParameters,
        user_message: String,
    },
    #[serde(rename = "log_expense")]
    LogExpense {
        params: tools_model::LogExpenseParameters,
        user_message: String,
    },
    #[serde(rename = "get_latest_media_context")]
    GetLatestMediaContext {
        params: GetLatestMediaContextParameters,
        user_message: String,
    },
}

#[derive(Clone)]
pub struct ToolDispatcher {
    pool: Pool<Postgres>,
    workspace_root: PathBuf,
    user_id: Option<Uuid>,
    conversation_id: Option<Uuid>,
    gemini: Arc<gemini_rust::Gemini>,
    gemini_api_key: String,
    sse: Arc<crate::common::sse::sse_emitter::SseBroadcaster>,
    storage: crate::common::storage::StorageClient,
}

impl ToolDispatcher {
    pub fn new(
        pool: Pool<Postgres>,
        workspace_root: PathBuf,
        user_id: Option<Uuid>,
        conversation_id: Option<Uuid>,
        gemini: Arc<gemini_rust::Gemini>,
        gemini_api_key: String,
        sse: Arc<crate::common::sse::sse_emitter::SseBroadcaster>,
        storage: crate::common::storage::StorageClient,
    ) -> Self {
        Self {
            pool,
            workspace_root,
            user_id,
            conversation_id,
            gemini,
            gemini_api_key,
            sse,
            storage,
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
            ArtaTool::ParseStringToJson { .. } => ToolResult {
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
            ArtaTool::CreateReminder {
                params,
                user_message,
            } => self.create_reminder(params, user_message).await,
            ArtaTool::ModifyReminder {
                params,
                user_message,
            } => self.modify_reminder(params, user_message).await,
            ArtaTool::GetInboxSummary {
                params,
                user_message,
            } => self.get_inbox_summary(params, user_message).await,
            ArtaTool::GetReminderStats {
                params,
                user_message,
            } => self.get_reminder_stats(params, user_message).await,
            ArtaTool::SearchUsers {
                params,
                user_message,
            } => self.search_users(params, user_message).await,
            ArtaTool::UpdateUserProfile {
                params,
                user_message,
            } => self.update_user_profile(params, user_message).await,
            ArtaTool::SendDirectMessage {
                params,
                user_message,
            } => self.send_direct_message(params, user_message).await,
            ArtaTool::MakeSticker {
                params,
                user_message,
            } => self.make_sticker(params, user_message).await,
            ArtaTool::LogExpense {
                params,
                user_message,
            } => self.log_expense(params, user_message).await,
            ArtaTool::GetLatestMediaContext {
                params,
                user_message,
            } => self.get_latest_media_context(params, user_message).await,
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

        let create_reminder = FunctionDeclaration::new(
            "create_reminder",
            "Schedule a new reminder for the user. Supports natural language descriptions and recurrence (daily, weekly, monthly). ALWAYS use the format YYYY-MM-DDTHH:MM:SSZ for due_at. Do not use milliseconds or offsets unless explicitly required. Always convert relative times (e.g., 'in 2 minutes') into an absolute ISO 8601 UTC timestamp based on the current time provided in the system prompt.",
            None,
        )
            .with_parameters::<CreateReminderParameters>()
            .with_response::<CreateReminderResponse>();

        let modify_reminder = FunctionDeclaration::new(
            "modify_reminder",
            "Modify an existing reminder: snooze it to a new time, cancel it, or mark it as done.",
            None,
        )
        .with_parameters::<ModifyReminderParameters>()
        .with_response::<ModifyReminderResponse>();

        let get_inbox_summary = FunctionDeclaration::new(
            "get_inbox_summary",
            "Retrieves a summary of recent messages from users. Use this when User asks: 'Any new DMs?', 'Who messaged me?', or 'Are there any strangers?'",
            None,
        )
            .with_parameters::<GetInboxSummaryParameters>()
            .with_response::<GetInboxSummaryResponse>();

        let get_reminder_stats = FunctionDeclaration::new(
            "get_reminder_stats",
            "Get stats about existing reminders, optionally filtered by DateTime ranges. Examples: 'What's left for the rest of the day?', 'Any reminders for this weekend?'",
            None,
        )
            .with_parameters::<GetReminderStatsParameters>()
            .with_response::<GetReminderStatsResponse>();

        let search_users = FunctionDeclaration::new(
            "search_users",
            "Searches the users table across username, display_name, and email using a case-insensitive partial match.",
            None,
        )
            .with_parameters::<SearchUsersParameters>()
            .with_response::<SearchUsersResponse>();

        let update_user_profile = FunctionDeclaration::new(
            "update_user_profile",
            "Allows updating the display_name of the current user. Restricted to the user_id extracted from the current session/JWT.",
            None,
        )
            .with_parameters::<UpdateUserProfileParameters>()
            .with_response::<UpdateUserProfileResponse>();

        let send_direct_message = FunctionDeclaration::new(
            "send_direct_message",
            "Sends a direct message to another user. Use search_users first to find their correct JID (user ID). Provide the recipient_jid and the message content.",
            None,
        )
            .with_parameters::<SendDirectMessageParameters>()
            .with_response::<SendDirectMessageResponse>();

        let make_sticker = FunctionDeclaration::new(
            "make_sticker",
            "Turns an image into a sticker. If no image_url is provided, it will use the most recently uploaded image in the conversation.",
            None,
        )
            .with_parameters::<MakeStickerParameters>()
            .with_response::<MakeStickerResponse>();

        let log_expense = FunctionDeclaration::new(
            "log_expense",
            "Log a financial expense. DO NOT guess or hallucinate item names (like Lorem Ipsum) or prices. If data is missing or unclear, DO NOT use dummy data; instead, ask the user for clarification.",
            None,
        )
            .with_parameters::<tools_model::LogExpenseParameters>()
            .with_response::<tools_model::LogExpenseResponse>();

        let get_latest_media_context = FunctionDeclaration::new(
            "get_latest_media_context",
            "Retrieve the latest media (image, video, etc.) context from the current conversation if not provided in the current turn.",
            None,
        )
            .with_parameters::<GetLatestMediaContextParameters>()
            .with_response::<GetLatestMediaContextResponse>();

        Tool::with_functions(vec![
            read_workspace_file,
            execute_read_query,
            web_search,
            read_web_page,
            update_nomi_soul,
            update_knowledge_base,
            evolve_bootstrap_content,
            create_reminder,
            modify_reminder,
            get_inbox_summary,
            get_reminder_stats,
            search_users,
            update_user_profile,
            send_direct_message,
            make_sticker,
            log_expense,
            get_latest_media_context,
        ])
    }

    async fn create_reminder(
        &self,
        params: CreateReminderParameters,
        _user_message: String,
    ) -> ToolResult {
        info!("Creating reminder: {}", params.description);

        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "User ID not found in context".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let due_at = match chrono::DateTime::parse_from_rfc3339(&params.due_at) {
            Ok(dt) => dt.with_timezone(&chrono::Utc),
            Err(e) => {
                return ToolResult {
                    error: format!("Invalid date format: {}. Please use ISO 8601.", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let frequency = params.frequency.unwrap_or_else(|| "once".to_string());

        let result = sqlx::query!(
            "INSERT INTO reminders (user_id, conversation_id, content, due_at, frequency, max_repeats) 
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            user_id,
            self.conversation_id,
            params.description,
            due_at,
            frequency,
            params.max_repeats
        )
            .fetch_one(&self.pool)
            .await;

        info!("Created reminder: {:?}", result);
        match result {
            Ok(row) => ToolResult {
                error: "".to_string(),
                success: true,
                content: format!("Reminder created successfully with ID: {}", row.id),
                follow_up_prompt: "".to_string(),
            },
            Err(e) => ToolResult {
                error: format!("Failed to create reminder: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn modify_reminder(
        &self,
        params: ModifyReminderParameters,
        _user_message: String,
    ) -> ToolResult {
        info!(
            "Modifying reminder: {} with action: {}",
            params.reminder_id, params.action
        );

        let reminder_id = match Uuid::parse_str(&params.reminder_id) {
            Ok(id) => id,
            Err(e) => {
                return ToolResult {
                    error: format!("Invalid reminder ID: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let result = match params.action.as_str() {
            "done" | "completed" => {
                sqlx::query!(
                    "UPDATE reminders SET status = 'completed', updated_at = NOW() WHERE id = $1",
                    reminder_id
                )
                .execute(&self.pool)
                .await
            }
            "cancel" | "archived" => {
                sqlx::query!(
                    "UPDATE reminders SET status = 'archived', updated_at = NOW() WHERE id = $1",
                    reminder_id
                )
                .execute(&self.pool)
                .await
            }
            "snooze" => {
                let snooze_until = match params.snooze_until {
                    Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                        Ok(dt) => dt.with_timezone(&chrono::Utc),
                        Err(e) => {
                            return ToolResult {
                                error: format!(
                                    "Invalid snooze date format: {}. Please use ISO 8601.",
                                    e
                                ),
                                success: false,
                                content: "".to_string(),
                                follow_up_prompt: "".to_string(),
                            };
                        }
                    },
                    None => {
                        return ToolResult {
                            error: "Snooze action requires 'snooze_until' parameter.".to_string(),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: "".to_string(),
                        };
                    }
                };

                sqlx::query!(
                    "UPDATE reminders SET due_at = $1, status = 'pending', snooze_count = snooze_count + 1, updated_at = NOW() WHERE id = $2",
                    snooze_until,
                    reminder_id
                )
                    .execute(&self.pool)
                    .await
            }
            _ => {
                return ToolResult {
                    error: format!("Invalid action: {}", params.action),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        match result {
            Ok(_) => ToolResult {
                error: "".to_string(),
                success: true,
                content: format!("Reminder {} successfully.", params.action),
                follow_up_prompt: "".to_string(),
            },
            Err(e) => ToolResult {
                error: format!("Failed to modify reminder: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
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
                info!("TAVILY_API_KEY not found in environment");
                return ToolResult {
                    error: "Cannot reach website search".to_string(),
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
            Err(e) => {
                info!("Error execute tavily: {}", e);
                ToolResult {
                    error: format!("Web search API error: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                }
            }
        }
    }

    async fn read_web_page(&self, url: String, user_message: String) -> ToolResult {
        info!(url = %url, "Executing read_web_page via Jina Reader");

        let client = reqwest::Client::new();
        let jina_url = format!("https://r.jina.ai/{}", url);

        let api_key = match std::env::var("JINA_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                info!("JINA_API_KEY not found in environment");
                return ToolResult {
                    error: "Failed read web page".to_string(),
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
            Err(e) => {
                info!("Error execute jina: {}", e);
                ToolResult {
                    error: format!("Web Reader error: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                }
            }
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
        // Intent-Media Linking
        let image_url = if let Some(url) = params.image_url {
            Some(self.storage.get_full_url(&url))
        } else if let Some(conv_id) = self.conversation_id {
            match crate::common::repository::pending_media_repo::get_pending_media(
                &self.pool, conv_id,
            )
            .await
            {
                Ok(Some(media)) => Some(media.media_url),
                _ => None,
            }
        } else {
            None
        };

        let summarizer_prompt =
            PromptRegistry::memory_consolidation_summarizer(params.content.as_str());

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
                "image_url": image_url,
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
                self.conversation_id,
            )
            .await;

            match save_result {
                Ok(_) => {
                    // Cleanup: Clear pending media from table
                    if let Some(conv_id) = self.conversation_id {
                        let _ =
                            crate::common::repository::pending_media_repo::delete_pending_media(
                                &self.pool, conv_id,
                            )
                            .await;
                    }

                    let msg = format!(
                        "Successfully saved to knowledge base: {}. Linked image cleared from pending queue.",
                        params.category
                    );
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
    async fn get_inbox_summary(
        &self,
        params: GetInboxSummaryParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Executing get_inbox_summary");

        let limit = params.limit.unwrap_or(5) as i64;
        let only_strangers = params.only_strangers.unwrap_or(false);

        // Security: Ensure only Trian can call this.
        // Assuming Trian is the only admin, we can check role,
        // or just rely on the fact that Trian's user_id is the one we want to exclude from the inbox itself.
        // The prompt says "Ensure this tool is only accessible when the requester is Trian (check sender_id)."
        // Let's get the user_id of the requester to exclude them from the result,
        // assuming the requester IS Trian.
        if let None = self.user_id {
            info!("user id not found");
            return ToolResult {
                error: "User ID not found in context. Cannot verify identity.".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }
        let admin_id = self.user_id.clone();

        // If we want to strictly check "is Trian", we'd check their role in DB.
        let is_admin: Result<bool, sqlx::Error> =
            sqlx::query_scalar("SELECT role = 'admin' FROM users WHERE id = $1")
                .bind(admin_id)
                .fetch_one(&self.pool)
                .await;

        if let Err(err) = is_admin {
            info!("user id not found {}", err);
            return ToolResult {
                error: format!("Failed to verify identity: {}", err),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }

        if let Ok(is_admin) = is_admin {
            info!("is user admin {}", is_admin);
            if !is_admin {
                return ToolResult {
                    error: "Unauthorized: Only Trian can use this tool.".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt:
                        "Tell Trian that someone unauthorized tried to check his DMs.".to_string(),
                };
            }
        }

        // We use a struct to hold the record to keep it lean.
        #[derive(serde::Serialize)]
        struct InboxRow {
            conversation_id: Option<Uuid>,
            display_name: Option<String>,
            phone_number: String,
            last_message: String,
            created_at: Option<chrono::DateTime<chrono::Utc>>,
            is_verified: Option<bool>,
        }

        let get_data = sqlx::query_as!(
            InboxRow,
            r#"
                SELECT
                    c.id as "conversation_id?",
                    u.display_name as "display_name?",
                    u.external_id as "phone_number!",
                    m.content as "last_message!",
                    m.created_at as "created_at?",
                    COALESCE(u.is_verified, false) as "is_verified?"
                FROM messages m
                JOIN conversations c ON m.conversation_id = c.id
                JOIN users u ON m.user_id = u.id
                WHERE u.id != $1
                AND m.role = 'user'
                AND ($3 = false OR COALESCE(u.is_verified, false) = false)
                AND m.id IN (
                    SELECT (
                        SELECT m2.id FROM messages m2
                        WHERE m2.conversation_id = m.conversation_id
                        ORDER BY m2.created_at DESC LIMIT 1
                    )
                )
                ORDER BY m.created_at DESC
                LIMIT $2;
                "#,
            admin_id,
            limit,
            only_strangers
        )
        .fetch_all(&self.pool)
        .await;

        if let Err(err) = get_data {
            info!("Error getting inbox rows: {}", err);
            return ToolResult {
                error: format!("Database error fetching inbox: {}", err),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "Failed to read messages due to an internal error.".to_string(),
            };
        }

        let rows = get_data.unwrap();

        let content = if rows.is_empty() {
            "No recent messages found.".to_string()
        } else {
            serde_json::to_string_pretty(&rows).unwrap_or_default()
        };

        ToolResult {
            error: "".to_string(),
            success: true,
            content: content.clone(),
            follow_up_prompt: build_follow_up_prompt(
                user_message,
                content,
                "get_inbox_summary".to_string(),
            ),
        }
    }

    async fn get_reminder_stats(
        &self,
        params: GetReminderStatsParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Executing get_reminder_stats");

        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "User ID not found in context".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let start_after = match params.start_after {
            Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                Err(e) => {
                    return ToolResult {
                        error: format!("Invalid start_after format: {}. Please use ISO 8601.", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    };
                }
            },
            None => None,
        };

        let end_before = match params.end_before {
            Some(ref s) => match chrono::DateTime::parse_from_rfc3339(s) {
                Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                Err(e) => {
                    return ToolResult {
                        error: format!("Invalid end_before format: {}. Please use ISO 8601.", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    };
                }
            },
            None => None,
        };

        let limit = params.limit.unwrap_or(20) as i64;

        let query_result = sqlx::query!(
            r#"
            SELECT 
                id,
                content,
                due_at,
                status,
                frequency,
                current_runs
            FROM reminders
            WHERE user_id = $1
              AND ($2::TIMESTAMPTZ IS NULL OR due_at >= $2)
              AND ($3::TIMESTAMPTZ IS NULL OR due_at <= $3)
              AND ($4::TEXT IS NULL OR status = $4)
            ORDER BY due_at ASC
            LIMIT $5;
            "#,
            user_id,
            start_after,
            end_before,
            params.status_filter,
            limit
        )
        .fetch_all(&self.pool)
        .await;

        match query_result {
            Ok(rows) => {
                let mut results = Vec::new();
                for row in rows {
                    let item = json!({
                        "id": row.id.to_string(),
                        "content": row.content,
                        "due_at": row.due_at.to_rfc3339(),
                        "status": row.status,
                        "frequency": row.frequency,
                        "current_runs": row.current_runs
                    });
                    results.push(item);
                }

                let content = if results.is_empty() {
                    "No reminders found for the given criteria.".to_string()
                } else {
                    serde_json::to_string_pretty(&results).unwrap_or_default()
                };

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "get_reminder_stats".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error fetching reminders: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn publish_to_nomi_outbond(&self, text: &str) {
        if let Some(conv_id) = self.conversation_id {
            let channel_info = sqlx::query!(
                "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
                conv_id
            ).fetch_all(&self.pool).await.unwrap_or_default();

            if let Ok(redis_url) = std::env::var("REDIS_URL") {
                if let Ok(client) = redis::Client::open(redis_url) {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        use redis::AsyncCommands;
                        for channel in channel_info {
                            let payload = serde_json::json!({
                                "is_group": false,
                                "sender_id": channel.external_id,
                                "conversation_id": channel.external_chat_id,
                                "text": text,
                                "channel": channel.channel_type,
                                "video_url": None::<String>,
                                "image_url": None::<String>,
                                "audio_url": None::<String>,
                                "doc_url": None::<String>,
                                "sticker_url": None::<String>,
                                "metadata": None::<serde_json::Value>,
                            })
                            .to_string();
                            let _ = conn
                                .publish::<&str, String, ()>("nomi:outbound", payload)
                                .await;
                        }
                    }
                }
            }
        }
    }

    async fn search_users(
        &self,
        params: SearchUsersParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Searching users for query: {}", params.query);
        let pattern = format!("%{}%", params.query);
        // We use 'name' column for username based on the schema.
        let results = sqlx::query!(
            "SELECT id, name as username, display_name, email FROM users \
             WHERE name ILIKE $1 OR display_name ILIKE $1 OR email ILIKE $1 LIMIT 20",
            pattern
        )
        .fetch_all(&self.pool)
        .await;

        match results {
            Ok(rows) => {
                if rows.is_empty() {
                    return ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: "No users found".to_string(),
                        follow_up_prompt: build_follow_up_prompt(
                            user_message,
                            "No users found".to_string(),
                            "search_users".to_string(),
                        ),
                    };
                }

                let mut summary = String::new();
                for row in rows {
                    summary.push_str(&format!(
                        "- ID: {}, Username: {}, Display: {}, Email: {}\n",
                        row.id,
                        row.username.as_deref().unwrap_or("N/A"),
                        row.display_name.as_deref().unwrap_or("N/A"),
                        row.email.as_deref().unwrap_or("N/A")
                    ));
                }

                let content = format!("Found {} users:\n{}", summary.lines().count(), summary);
                self.publish_to_nomi_outbond(&format!(
                    "Searched for users matching '{}'",
                    params.query
                ))
                .await;

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "search_users".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error searching users: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn update_user_profile(
        &self,
        params: UpdateUserProfileParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Updating user profile");

        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "User ID not found in context".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let result = sqlx::query!(
            "UPDATE users SET display_name = $1 WHERE id = $2",
            params.display_name,
            user_id
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                let content = format!(
                    "Successfully updated display_name to '{}'",
                    params.display_name
                );
                self.publish_to_nomi_outbond(&content).await;

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "update_user_profile".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error updating profile: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn log_expense(
        &self,
        params: tools_model::LogExpenseParameters,
        user_message: String,
    ) -> ToolResult {
        let user_id = match self.user_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "User not authenticated".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        if let None = self.conversation_id {
            info!("Logging user {} to expense but conversation id is null", user_id);
            return ToolResult {
                error: "Conversation ID not found".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }

        let expense_data = crate::feature::message_processor::media::ExpenseData {
            merchant: params.merchant,
            total: params.total,
            tax: params.tax,
            service: params.service,
            discount: params.discount,
            items: params
                .items
                .into_iter()
                .map(|i| crate::feature::message_processor::media::ExpenseItem {
                    name: i.name,
                    quantity: i.quantity,
                    amount: i.amount,
                })
                .collect(),
            category: params.category,
        };

        match crate::feature::message_processor::processor::log_expense_transaction(
            &self.pool,
            user_id,
            self.conversation_id,
            &expense_data,
        )
        .await
        {
            Ok(_) => {
                let content = format!(
                    "Expense of {} at {} logged successfully under {}. Attached image linked and cleared from pending queue.",
                    expense_data.total, expense_data.merchant, expense_data.category
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "log_expense".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Failed to log expense: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn make_sticker(
        &self,
        params: MakeStickerParameters,
        user_message: String,
    ) -> ToolResult {
        let conversation_id = match self.conversation_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "Conversation ID not found in context".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let image_url = if let Some(url) = params.image_url {
            self.storage.get_full_url(&url)
        } else {
            // Retrieve from pending_media table
            match crate::common::repository::pending_media_repo::get_pending_media(
                &self.pool,
                conversation_id,
            )
            .await
            {
                Ok(Some(media)) => media.media_url,
                Ok(None) => {
                    return ToolResult {
                            error: "No recent image found to turn into a sticker. Please upload an image first!".to_string(),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: "".to_string(),
                        };
                }
                Err(e) => {
                    return ToolResult {
                        error: format!("Database error: {}", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    };
                }
            }
        };

        info!("Generating sticker for path: {}", image_url);

        // Find channels for this conversation
        let channel_info = sqlx::query!(
            "SELECT c.channel_type, c.external_id, c.external_chat_id FROM channels c JOIN conversation_members cm ON c.user_id = cm.user_id WHERE cm.conversation_id = $1",
            conversation_id
        ).fetch_all(&self.pool).await.unwrap_or_default();

        if channel_info.is_empty() {
            return ToolResult {
                error: "No active channels found for this conversation".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }

        for channel in channel_info {
            let outbound = crate::feature::OutboundMessage {
                is_group: false,
                sender_id: channel.external_id.clone(),
                conversation_id: channel.external_chat_id.clone(),
                text: "Coming up! 🚀".to_string(),
                channel: channel.channel_type.clone(),
                video_url: None,
                image_url: None,
                audio_url: None,
                doc_url: None,
                sticker_url: Some(image_url.clone()),
                metadata: None,
            };
            let _ = self.redis_publish_outbound(&outbound).await;
        }

        // Cleanup: Clear pending media from table
        let _ = crate::common::repository::pending_media_repo::delete_pending_media(
            &self.pool,
            conversation_id,
        )
        .await;

        let content = "Sticker generation triggered! 🚀 (Linked image cleared from pending queue)"
            .to_string();
        ToolResult {
            error: "".to_string(),
            success: true,
            content: content.clone(),
            follow_up_prompt: build_follow_up_prompt(
                user_message,
                content,
                "make_sticker".to_string(),
            ),
        }
    }

    async fn send_direct_message(
        &self,
        params: SendDirectMessageParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Sending direct message to: {}", params.recipient_jid);

        // We need to find a channel for the recipient to know where to send it.
        // For simplicity, we'll pick the most recent channel for that user.
        let channel_info = sqlx::query!(
            "SELECT channel_type, external_id, external_chat_id FROM channels WHERE external_id = $1 OR user_id::text = $1 ORDER BY created_at DESC LIMIT 1",
            params.recipient_jid
        ).fetch_optional(&self.pool).await;

        match channel_info {
            Ok(Some(channel)) => {
                let outbound = crate::feature::OutboundMessage {
                    is_group: false,
                    sender_id: channel.external_id.clone(),
                    conversation_id: channel.external_chat_id.clone(),
                    text: params.content.clone(),
                    channel: channel.channel_type.clone(),
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: None,
                };

                // Publish to Redis using the helper method or direct client
                let _ = self.redis_publish_outbound(&outbound).await;

                let content = format!(
                    "Message sent to {}: {}",
                    params.recipient_jid, params.content
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "send_direct_message".to_string(),
                    ),
                }
            }
            Ok(None) => ToolResult {
                error: format!("No active channel found for user {}", params.recipient_jid),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
            Err(e) => ToolResult {
                error: format!("Database error looking up recipient: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn get_latest_media_context(
        &self,
        _params: GetLatestMediaContextParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Retrieving latest media context");
        let conversation_id = match self.conversation_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "No active conversation context".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        match crate::common::repository::pending_media_repo::get_pending_media(
            &self.pool,
            conversation_id,
        )
        .await
        {
            Ok(Some(media)) => {
                let full_url = self.storage.get_full_url(&media.media_url);
                let content = format!(
                    "Latest media in conversation: URL: {}, Type: {}, Classification: {}, Created At: {}",
                    full_url,
                    media.media_type,
                    media.classification.as_deref().unwrap_or("None"),
                    media.created_at
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: content.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        content,
                        "get_latest_media_context".to_string(),
                    ),
                }
            }
            Ok(None) => ToolResult {
                error: "".to_string(),
                success: true,
                content: "No pending media found for this conversation.".to_string(),
                follow_up_prompt: "".to_string(),
            },
            Err(e) => ToolResult {
                error: format!("Database error: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn redis_publish_outbound(
        &self,
        outbound: &crate::feature::OutboundMessage,
    ) -> anyhow::Result<()> {
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            let client = crate::common::redis::RedisClient::new(&redis_url)?;
            client.publish_event("nomi:channel", outbound).await?;
        }
        Ok(())
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
