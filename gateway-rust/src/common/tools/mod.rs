pub mod plugin_trait;
pub mod plugins;
pub mod tools_model;

use crate::Arc;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::plugins::analyze_media::AnalyzeMediaPlugin;
use crate::common::tools::plugins::communication::CommunicationPlugin;
use crate::common::tools::plugins::dice::DicePlugin;
use crate::common::tools::plugins::evolve_bootstrap::EvolveBootstrapPlugin;
use crate::common::tools::plugins::execute_sql_query::ExecuteSqlQueryPlugin;
use crate::common::tools::plugins::finance::FinancePlugin;
use crate::common::tools::plugins::get_inbox_summary::GetInboxSummaryPlugin;
use crate::common::tools::plugins::get_latest_media_context::GetLatestMediaContextPlugin;
use crate::common::tools::plugins::get_reminder_stats::GetReminderStatsPlugin;
use crate::common::tools::plugins::health::HealthPlugin;
use crate::common::tools::plugins::modify_reminder::ModifyReminderPlugin;
use crate::common::tools::plugins::parse_to_json::ParseStringToJsonPlugin;
use crate::common::tools::plugins::read_web_page::ReadWebPagePlugin;
use crate::common::tools::plugins::read_workspace_file::ReadWorkspaceFilePlugin;
use crate::common::tools::plugins::retrieve_knowledge::RetrieveKnowledgePlugin;
use crate::common::tools::plugins::schedule_task::ScheduleTaskPlugin;
use crate::common::tools::plugins::sticker_generator::StickerGeneratorPlugin;
use crate::common::tools::plugins::update_conversation_soul::UpdateConversationSoulPlugin;
use crate::common::tools::plugins::update_conversation_title::UpdateConversationTitlePlugin;
use crate::common::tools::plugins::update_knowledge::UpdateKnowledgeBasePlugin;
use crate::common::tools::plugins::user::UserPlugin;
use crate::common::tools::plugins::web_search::WebSearchPlugin;
use gemini_rust::{FunctionDeclaration, Tool};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use crate::common::tools::plugins::forecast::WeatherFallbackPlugin;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "tool", content = "args")]
pub enum NomiTool {}

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
        plugins.insert("update_knowledge_base", Arc::new(UpdateKnowledgeBasePlugin));
        plugins.insert("retrieve_knowledge", Arc::new(RetrieveKnowledgePlugin));
        plugins.insert(
            "update_conversation_title",
            Arc::new(UpdateConversationTitlePlugin),
        );
        plugins.insert("update_nomi_soul", Arc::new(UpdateConversationSoulPlugin));
        plugins.insert("evolve_bootstrap_content", Arc::new(EvolveBootstrapPlugin));
        plugins.insert("analyze_media", Arc::new(AnalyzeMediaPlugin));
        plugins.insert(
            "get_latest_media_context",
            Arc::new(GetLatestMediaContextPlugin),
        );
        plugins.insert("get_inbox_summary", Arc::new(GetInboxSummaryPlugin));
        plugins.insert("execute_read_query", Arc::new(ExecuteSqlQueryPlugin));
        plugins.insert("read_workspace_file", Arc::new(ReadWorkspaceFilePlugin));
        plugins.insert("parse_to_json", Arc::new(ParseStringToJsonPlugin));
        plugins.insert("create_sticker", Arc::new(StickerGeneratorPlugin));
        plugins.insert("get_current_weather", Arc::new(WeatherFallbackPlugin));

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
    

    pub fn generate_tool_for_prompt(&self, intents: &[String]) -> Tool {
        let mut tools = Vec::new();

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
