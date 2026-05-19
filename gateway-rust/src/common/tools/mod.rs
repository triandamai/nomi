pub mod plugin_trait;
pub mod plugins;
pub mod tools_model;

use crate::Arc;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::plugins::communication::CommunicationPlugin;
use crate::common::tools::plugins::dice::DicePlugin;
use crate::common::tools::plugins::finance::FinancePlugin;
use crate::common::tools::plugins::get_reminder_stats::GetReminderStatsPlugin;
use crate::common::tools::plugins::health::HealthPlugin;
use crate::common::tools::plugins::modify_reminder::ModifyReminderPlugin;
use crate::common::tools::plugins::read_web_page::ReadWebPagePlugin;
use crate::common::tools::plugins::schedule_task::ScheduleTaskPlugin;
use crate::common::tools::plugins::user::UserPlugin;
use crate::common::tools::plugins::web_search::WebSearchPlugin;
use crate::common::tools::tools_model::{
    EvolveBootstrapParameters, EvolveBootstrapResponse, ExecuteReadQueryParameters,
    ExecuteReadQueryResponse, GetInboxSummaryParameters, GetInboxSummaryResponse,
    GetLatestMediaContextParameters, MakeStickerParameters, MakeStickerResponse,
    ParseToJsonParameters, ReadWorkSpaceParameters, ReadWorkSpaceResponse, ToolResult,
    UpdateConversationSoulParameters, UpdateConversationSoulResponse,
    UpdateConversationTitleParameters, UpdateConversationTitleResponse,
    UpdateKnowledgeBaseParameters, UpdateKnowledgeBaseResponse,
};
use crate::prompts::PromptRegistry;
use chrono_tz::Tz;
use dotenvy::var;
use gemini_rust::{FunctionDeclaration, Tool, UsageMetadata};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sqlx::{Column, Pool, Postgres, Row};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "tool", content = "args")]
pub enum NomiTool {
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
    #[serde(rename = "retrieve_knowledge")]
    RetrieveKnowledge {
        params: tools_model::RetrieveKnowledgeParameters,
        user_message: String,
    },
    #[serde(rename = "evolve_bootstrap_content")]
    EvolveBootstrap {
        params: EvolveBootstrapParameters,
        user_message: String,
    },
    #[serde(rename = "get_inbox_summary")]
    GetInboxSummary {
        params: GetInboxSummaryParameters,
        user_message: String,
    },
    #[serde(rename = "make_sticker")]
    MakeSticker {
        params: MakeStickerParameters,
        user_message: String,
    },
    #[serde(rename = "get_latest_media_context")]
    GetLatestMediaContext {
        params: GetLatestMediaContextParameters,
        user_message: String,
    },
    #[serde(rename = "analyze_media")]
    AnalyzeMedia {
        params: tools_model::AnalyzeMediaParameters,
        user_message: String,
    },
    #[serde(rename = "update_conversation_title")]
    UpdateConversationTitle {
        params: UpdateConversationTitleParameters,
        user_message: String,
    },
}

#[derive(Clone)]
pub struct ToolDispatcher {
    pub pool: Pool<Postgres>,
    pub workspace_root: PathBuf,
    pub user_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub gemini: Arc<gemini_rust::Gemini>,
    pub gemini_api_key: String,
    pub storage: crate::common::storage::StorageClient,
    pub app_state: crate::common::app_state::AppState,
    pub plugins: HashMap<&'static str, Arc<dyn NomiToolPlugin>>,
}

impl ToolDispatcher {
    pub fn new(
        pool: Pool<Postgres>,
        workspace_root: PathBuf,
        user_id: Option<Uuid>,
        conversation_id: Option<Uuid>,
        gemini: Arc<gemini_rust::Gemini>,
        gemini_api_key: String,
        storage: crate::common::storage::StorageClient,
        app_state: crate::common::app_state::AppState,
    ) -> Self {
        let mut plugins: HashMap<&'static str, Arc<dyn NomiToolPlugin>> = HashMap::new();
        plugins.insert("roll_dice", Arc::new(DicePlugin));
        plugins.insert("manage_health_data", Arc::new(HealthPlugin));
        plugins.insert("manage_finance", Arc::new(FinancePlugin));
        plugins.insert("manage_user", Arc::new(UserPlugin));
        plugins.insert("send_message", Arc::new(CommunicationPlugin));
        plugins.insert("web_search", Arc::new(WebSearchPlugin));
        plugins.insert("read_web_page", Arc::new(ReadWebPagePlugin));
        plugins.insert("schedule_task", Arc::new(ScheduleTaskPlugin));
        plugins.insert("modify_reminder", Arc::new(ModifyReminderPlugin));
        plugins.insert("get_reminder_stats", Arc::new(GetReminderStatsPlugin));

        Self {
            pool,
            workspace_root,
            user_id,
            conversation_id,
            gemini,
            gemini_api_key,
            storage,
            app_state,
            plugins,
        }
    }

    pub async fn dispatch(&self, tool: NomiTool) -> ToolResult {
        match tool {
            NomiTool::ReadWorkspaceFile {
                params,
                user_message,
            } => self.read_workspace_file(params.path, user_message).await,
            NomiTool::ExecuteSqlQuery {
                params,
                user_message,
            } => self.execute_sql_query(params.query, user_message).await,
            NomiTool::ParseStringToJson { .. } => ToolResult {
                error: "".to_string(),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
            NomiTool::UpdateConversationSoul {
                params,
                user_message,
            } => self.update_nomi_soul(params, user_message).await,
            NomiTool::UpdateKnowledgeBase {
                params,
                user_message,
            } => self.update_knowledge_base(params, user_message).await,
            NomiTool::RetrieveKnowledge {
                params,
                user_message,
            } => self.retrieve_knowledge(params, user_message).await,
            NomiTool::EvolveBootstrap {
                params,
                user_message,
            } => self.evolve_bootstrap(params, user_message).await,
            NomiTool::GetInboxSummary {
                params,
                user_message,
            } => self.get_inbox_summary(params, user_message).await,
            NomiTool::MakeSticker {
                params,
                user_message,
            } => self.make_sticker(params, user_message).await,
            NomiTool::GetLatestMediaContext {
                params,
                user_message,
            } => self.get_latest_media_context(params, user_message).await,
            NomiTool::AnalyzeMedia {
                params,
                user_message,
            } => self.analyze_media(params, user_message).await,
            NomiTool::UpdateConversationTitle {
                params,
                user_message,
            } => self.update_conversation_title(params, user_message).await,
            // _ => ToolResult {
            //     error: "No  tool match".to_string(),
            //     success: false,
            //     content: "".to_string(),
            //     follow_up_prompt: "".to_string(),
            // },
        }
    }

    pub fn generate_tool_for_prompt(&self, intents: &[String]) -> Tool {
        let mut tools = Vec::new();

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

        let retrieve_knowledge =
            FunctionDeclaration::new(
                "retrieve_knowledge",
                "Search your long-term memory for specific facts, preferences, and project details. Use start_date and end_date (ISO 8601) if the query implies a timeframe (e.g., 'last week', 'yesterday', 'in March'). If general, leave them null.",
                None,
            )
                .with_parameters::<tools_model::RetrieveKnowledgeParameters>()
                .with_response::<tools_model::RetrieveKnowledgeResponse>();

        let evolve_bootstrap_content = FunctionDeclaration::new(
            "evolve_bootstrap_content",
            "Update your own personality or mission instructions (System Prompt) dynamically.",
            None,
        )
        .with_parameters::<EvolveBootstrapParameters>()
        .with_response::<EvolveBootstrapResponse>();

        let get_inbox_summary = FunctionDeclaration::new(
            "get_inbox_summary",
            "Retrieves a summary of recent messages from users. Use this when User asks: 'Any new DMs?', 'Who messaged me?', or 'Are there any strangers?'",
            None,
        )
            .with_parameters::<GetInboxSummaryParameters>()
            .with_response::<GetInboxSummaryResponse>();

        let make_sticker = FunctionDeclaration::new(
            "make_sticker",
            "Turns an image into a sticker. If no image_url is provided, it will use the most recently uploaded image in the conversation.",
            None,
        )
            .with_parameters::<MakeStickerParameters>()
            .with_response::<MakeStickerResponse>();

        let analyze_media = FunctionDeclaration::new(
            "analyze_media",
            "Analyze a media file (image, video, audio, or document) and provide information based on a prompt. Use this when the user asks questions about a file, wants to read text from it, or needs a description/summary. If no media_url is provided, it will use the most recently uploaded file in the conversation.",
            None,
        )
            .with_parameters::<tools_model::AnalyzeMediaParameters>()
            .with_response::<tools_model::AnalyzeMediaResponse>();

        let update_conversation_title = FunctionDeclaration::new(
            "update_conversation_title",
            "Updates the display title or topic name of the current conversation thread or group context inside the database dynamically.",
            None,
        )
            .with_parameters::<UpdateConversationTitleParameters>()
            .with_response::<UpdateConversationTitleResponse>();

        for intent in intents {
            match intent.as_str() {
                "FINANCE" => {
                    // Handled by FinancePlugin
                }
                "VITALITY" => {
                    // Handled by HealthPlugin
                }
                "STORAGE" => {
                    tools.push(update_knowledge_base.clone());
                    tools.push(retrieve_knowledge.clone());
                    tools.push(read_workspace_file.clone());
                    tools.push(execute_read_query.clone());
                }
                "REMINDER" => {
                    // Handled by Plugins
                }
                "WEB" => {
                    // Handled by Plugins
                    // Add Google Search retrieval if supported by model
                    let mut unique_tools = Vec::new();
                    let mut seen_names = std::collections::HashSet::new();
                    for t in tools {
                        if seen_names.insert(t.name.clone()) {
                            unique_tools.push(t);
                        }
                    }
                    return Tool::google_search();
                }
                "DASHBOARD" => {
                    tools.push(get_inbox_summary.clone());
                    tools.push(update_conversation_title.clone());
                    tools.push(retrieve_knowledge.clone());
                    tools.push(evolve_bootstrap_content.clone());
                    tools.push(update_nomi_soul.clone());
                }
                "COMMUNICATION" => {
                    tools.push(get_inbox_summary.clone());
                    tools.push(update_conversation_title.clone());
                }
                "GENERAL" => {
                    tools.push(update_conversation_title.clone());
                    tools.push(retrieve_knowledge.clone());
                    tools.push(evolve_bootstrap_content.clone());
                    tools.push(update_nomi_soul.clone());
                }
                _ => {}
            }
        }

        // Let's add some generic tools that might always be needed for intent != GENERAL, or specifically requested tools.
        // The instructions state: "Filter Tools: Pass only the tools relevant to the identified Intent to the LLM. Example: If FINANCE, only pass the expense tracking tools. Critical: If GENERAL, pass ZERO tools."
        // We'll also add some core ones or just rely strictly on the switch.

        // Wait, if fallback mode is triggered, intent could be set to "FULL_REGISTRY" or something.
        if intents.contains(&"FULL_REGISTRY".to_string()) {
            tools = vec![
                read_workspace_file,
                execute_read_query,
                update_nomi_soul,
                update_knowledge_base,
                retrieve_knowledge,
                evolve_bootstrap_content,
                get_inbox_summary,
                make_sticker,
                analyze_media,
                update_conversation_title,
            ];
        }

        // Plugable Tools Interception
        for plugin in self.plugins.values() {
            let plugin_intents = plugin.matching_intents();
            let is_matched = intents.iter().any(|i| plugin_intents.contains(&i.as_str()))
                || intents.contains(&"FULL_REGISTRY".to_string());

            if is_matched {
                let mut schema = plugin.schema();
                // STRICT ENFORCEMENT: Shield against Gemini API 400 validation drops by removing additionalProperties
                if let Some(obj) = schema.as_object_mut() {
                    if let Some(params) = obj.get_mut("parameters") {
                        if let Some(params_obj) = params.as_object_mut() {
                            params_obj.remove("additionalProperties");
                            // Recurse into properties if they exist
                            if let Some(properties) = params_obj.get_mut("properties") {
                                if let Some(props_obj) = properties.as_object_mut() {
                                    for (_, prop_val) in props_obj.iter_mut() {
                                        if let Some(prop_obj) = prop_val.as_object_mut() {
                                            prop_obj.remove("additionalProperties");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Ok(func_decl) = serde_json::from_value::<FunctionDeclaration>(schema) {
                    tools.push(func_decl);
                }
            }
        }

        // De-duplicate tools based on name if multiple intents added the same tool
        let mut unique_tools = Vec::new();
        let mut seen_names = std::collections::HashSet::new();
        for t in tools {
            if seen_names.insert(t.name.clone()) {
                unique_tools.push(t);
            }
        }

        Tool::with_functions(unique_tools)
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

                // Dispatch internal update
                let _ = self
                    .app_state
                    .dispatch(crate::services::event_dispatcher::AppEvent::broadcast(
                        "evolution",
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

    async fn retrieve_knowledge(
        &self,
        params: tools_model::RetrieveKnowledgeParameters,
        user_message: String,
    ) -> ToolResult {
        info!("Retrieving knowledge for query: {}", params.query);

        let start_date = params.start_date.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        let end_date = params.end_date.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });

        info!("Search from :{:?} => {:?}", start_date, end_date);
        let embedding_res = crate::rag::get_embedding(&self.gemini_api_key, &params.query).await;

        match embedding_res {
            Ok(embedding) => {
                let results = crate::utils::rag::hybrid_retrieve(
                    &self.pool,
                    &params.query,
                    embedding.embedding.values,
                    self.conversation_id,
                    start_date,
                    end_date,
                )
                .await;

                match results {
                    Ok(memories) => {
                        let content = memories.join("\n---\n");
                        ToolResult {
                            error: "".to_string(),
                            success: true,
                            content: content.clone(),
                            follow_up_prompt: build_follow_up_prompt(
                                user_message,
                                content,
                                "retrieve_knowledge".to_string(),
                            ),
                        }
                    }
                    Err(e) => ToolResult {
                        error: format!("Error retrieving knowledge: {}", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    },
                }
            }
            Err(e) => ToolResult {
                error: format!("Error generating embedding: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
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
            Ok(ref resp) => {
                let raw_json = resp.text();
                if let Some(start) = raw_json.find('{') {
                    if let Some(end) = raw_json.rfind('}') {
                        serde_json::from_str(&raw_json[start..=end]).unwrap_or(serde_json::json!({
                            "summary": params.content,
                            "nodes": [],
                            "edges": []
                        }))
                    } else {
                        json!({"summary": params.content, "nodes": [], "edges": []})
                    }
                } else {
                    json!({"summary": params.content, "nodes": [], "edges": []})
                }
            }
            Err(_) => {
                json!({"summary": params.content, "nodes": [], "edges": []})
            }
        };

        let summary_text = parsed_data["summary"]
            .as_str()
            .unwrap_or(&params.content)
            .to_string();

        if let Ok(embedding) = crate::rag::get_embedding(&self.gemini_api_key, &summary_text).await
        {
            let metadata = json!({
                "type": "memory",
                "category": params.category,
                "image_url": image_url,
                "graph": {
                    "nodes": parsed_data["nodes"],
                    "links": parsed_data["edges"]
                }
            });

            let usage = summary_res.map(|s| s.usage_metadata).map_or_else(
                |_| UsageMetadata {
                    prompt_token_count: None,
                    candidates_token_count: None,
                    total_token_count: None,
                    thoughts_token_count: None,
                    prompt_tokens_details: None,
                    cached_content_token_count: None,
                    cache_tokens_details: None,
                },
                |r| {
                    r.unwrap_or(UsageMetadata {
                        prompt_token_count: None,
                        candidates_token_count: None,
                        total_token_count: None,
                        thoughts_token_count: None,
                        prompt_tokens_details: None,
                        cached_content_token_count: None,
                        cache_tokens_details: None,
                    })
                },
            );
            let p_tokens = usage.prompt_token_count.unwrap_or(0);
            let a_tokens = usage.candidates_token_count.unwrap_or(0);
            let t_tokens = usage.total_token_count.unwrap_or(0);

            if let Ok(mut tx) = self.pool.begin().await {
                let save_result = crate::rag::save_to_knowledge_base(
                    &self.pool,
                    &summary_text,
                    embedding.embedding.values,
                    Some(metadata),
                    self.conversation_id,
                    p_tokens,
                    a_tokens,
                    t_tokens,
                )
                .await;

                match save_result {
                    Ok(_) => {
                        if let Some(conv_id) = self.conversation_id {
                            let updated_convo = sqlx::query!(
                                "UPDATE conversations SET cumulative_tokens = cumulative_tokens + $1 WHERE id = $2 RETURNING cumulative_tokens",
                                t_tokens,
                                conv_id
                            )
                                .fetch_one(&mut *tx)
                                .await;

                            if let Ok(row) = updated_convo {
                                // Dispatch token update
                                let _ = self
                                    .app_state
                                    .dispatch(
                                        crate::services::event_dispatcher::AppEvent::conversation(
                                            conv_id,
                                            "token_update",
                                            serde_json::json!({
                                                "conversation_id": conv_id,
                                                "cumulative_tokens": row.cumulative_tokens
                                            }),
                                        ),
                                    )
                                    .await;
                            }

                            // Cleanup: Clear pending media from table
                            let _ =
                                crate::common::repository::pending_media_repo::delete_pending_media(
                                    &self.pool, conv_id,
                                )
                                    .await;
                        }

                        let _ = tx.commit().await;

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

    async fn get_latest_media_context(
        &self,
        _params: GetLatestMediaContextParameters,
        _user_message: String,
    ) -> ToolResult {
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
                let tz_wib: Tz = "Asia/Jakarta".parse().unwrap();
                let created_at_wib = media.created_at.with_timezone(&tz_wib);
                let time_str = created_at_wib.format("%Y-%m-%d %H:%M WIB").to_string();

                let content = format!(
                    "I've retrieved the latest media from our 'Visual Buffer':\n\n\
                    - **Type:** {}\n\
                    - **Buffered At:** **{}**\n\
                    - **Status:** Pending Analysis 🔍\n\n\
                    What would you like me to do with this? I can log it as an expense, turn it into a sticker, or analyze its content for you! ✨",
                    media.media_type.to_uppercase(),
                    time_str
                );

                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content,
                    follow_up_prompt: "".to_string(),
                }
            }
            Ok(None) => ToolResult {
                error: "".to_string(),
                success: true,
                content: "Our 'Visual Buffer' is currently empty. No silent media has been captured recently! 🏔️"
                    .to_string(),
                follow_up_prompt: "".to_string(),
            },
            Err(e) => ToolResult {
                error: format!("Database error retrieving media: {}", e),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            },
        }
    }

    async fn analyze_media(
        &self,
        params: tools_model::AnalyzeMediaParameters,
        user_message: String,
    ) -> ToolResult {
        use base64::Engine;
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

        let media_url = if let Some(url) = params.media_url {
            url
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
                        error: "No recent image found to analyze. Please upload an image first!"
                            .to_string(),
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

        let base_url = var("PUBLIC_GATEWAY_URL").unwrap_or("http://localhost:8000/api".to_string());

        let image_url = if media_url.starts_with("http") && media_url.starts_with(base_url.as_str())
        {
            media_url.replace(format!("{}/files/", base_url).as_str(), "")
        } else {
            media_url.to_string()
        };

        if image_url.starts_with("http") {
            return ToolResult {
                error: format!("Tool doesnt support url from outside app"),
                success: false,
                content: "".to_string(),
                follow_up_prompt: "".to_string(),
            };
        }
        info!(
            "Analyzing image: {} with prompt: {}",
            image_url, params.prompt
        );

        let data = match self
            .storage
            .get_file("conversations".to_string(), image_url.clone())
            .await
        {
            Ok(d) => d,
            Err(e) => {
                return ToolResult {
                    error: format!("Storage error: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let mime_type = mime_guess::from_path(&image_url)
            .first_or_octet_stream()
            .to_string();

        let base64_data = base64::engine::general_purpose::STANDARD.encode(data.to_vec());

        let res = match self
            .gemini
            .generate_content()
            .with_message(gemini_rust::Message {
                role: gemini_rust::Role::User,
                content: gemini_rust::Content {
                    parts: Some(vec![
                        gemini_rust::Part::Text {
                            text: params.prompt.clone(),
                            thought: None,
                            thought_signature: None,
                        },
                        gemini_rust::Part::InlineData {
                            inline_data: gemini_rust::Blob {
                                mime_type,
                                data: base64_data,
                            },
                            media_resolution: None,
                        },
                    ]),
                    role: Some(gemini_rust::Role::User),
                },
            })
            .execute()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                return ToolResult {
                    error: format!("Gemini error: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let analysis = res.text();

        let response_data = tools_model::AnalyzeMediaResponse {
            content: analysis.clone(),
        };

        let content_json = serde_json::to_string_pretty(&response_data).unwrap_or_default();

        ToolResult {
            error: "".to_string(),
            success: true,
            content: content_json.clone(),
            follow_up_prompt: build_follow_up_prompt(
                user_message,
                content_json,
                "analyze_media".to_string(),
            ),
        }
    }

    async fn update_conversation_title(
        &self,
        params: UpdateConversationTitleParameters,
        user_message: String,
    ) -> ToolResult {
        let conversation_id = match self.conversation_id {
            Some(id) => id,
            None => {
                return ToolResult {
                    error: "No active conversation context found.".to_string(),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                };
            }
        };

        let result = sqlx::query!(
            "UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2",
            params.new_title,
            conversation_id
        )
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => {
                let msg = format!(
                    "Successfully changed workspace topic heading to '{}'",
                    params.new_title
                );
                ToolResult {
                    error: "".to_string(),
                    success: true,
                    content: msg.clone(),
                    follow_up_prompt: build_follow_up_prompt(
                        user_message,
                        msg,
                        "update_conversation_title".to_string(),
                    ),
                }
            }
            Err(e) => ToolResult {
                error: format!("Database error updating conversation title: {}", e),
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
