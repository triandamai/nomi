pub mod plugin_trait;
pub mod plugins;
pub mod tools_model;
pub mod edge_runner;

use crate::Arc;
use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::plugins::analyze_media::AnalyzeMediaPlugin;
use crate::common::tools::plugins::communication::CommunicationPlugin;
use crate::common::tools::plugins::discover_tools::DiscoverToolsPlugin;
use crate::common::tools::plugins::dice::DicePlugin;
use crate::common::tools::plugins::evolve_bootstrap::EvolveBootstrapPlugin;
use crate::common::tools::plugins::execute_sql_query::ExecuteSqlQueryPlugin;
use crate::common::tools::plugins::finance::FinancePlugin;
use crate::common::tools::plugins::get_inbox_summary::GetInboxSummaryPlugin;
use crate::common::tools::plugins::get_latest_media_context::GetLatestMediaContextPlugin;
use crate::common::tools::plugins::get_reminder_stats::GetReminderStatsPlugin;
use crate::common::tools::plugins::health::HealthPlugin;
use crate::common::tools::plugins::manage_proposals::ManageSkillProposalsPlugin;
use crate::common::tools::plugins::modify_reminder::ModifyReminderPlugin;
use crate::common::tools::plugins::parse_to_json::ParseStringToJsonPlugin;
use crate::common::tools::plugins::read_web_page::ReadWebPagePlugin;
use crate::common::tools::plugins::read_workspace_file::ReadWorkspaceFilePlugin;
use crate::common::tools::plugins::retrieve_knowledge::RetrieveKnowledgePlugin;
use crate::common::tools::plugins::schedule_task::ScheduleTaskPlugin;
use crate::common::tools::plugins::sticker_generator::StickerGeneratorPlugin;
use crate::common::tools::plugins::srp_summary::SrpSummaryPlugin;
use crate::common::tools::plugins::suggest_skill::SuggestSkillPlugin;
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
        plugins.insert("discover_tools", Arc::new(DiscoverToolsPlugin));
        plugins.insert("manage_health_data", Arc::new(HealthPlugin));
        plugins.insert("manage_finance", Arc::new(FinancePlugin));
        plugins.insert("manage_user", Arc::new(UserPlugin));
        plugins.insert("send_message", Arc::new(CommunicationPlugin));
        plugins.insert("web_search", Arc::new(WebSearchPlugin));
        plugins.insert("read_web_page", Arc::new(ReadWebPagePlugin));
        plugins.insert("schedule_task", Arc::new(ScheduleTaskPlugin));
        plugins.insert("modify_reminder", Arc::new(ModifyReminderPlugin));
        plugins.insert("manage_skill_proposals", Arc::new(ManageSkillProposalsPlugin));
        plugins.insert("get_srp_summary", Arc::new(SrpSummaryPlugin));
        plugins.insert("suggest_new_skill", Arc::new(SuggestSkillPlugin));
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
        plugins.insert("adjust_deb", Arc::new(crate::common::tools::plugins::adjust_deb::AdjustDebPlugin));
        plugins.insert("get_workspace_summary", Arc::new(crate::common::tools::plugins::get_workspace_summary::GetWorkspaceSummaryPlugin));
        plugins.insert("parse_to_json", Arc::new(ParseStringToJsonPlugin));
        plugins.insert("create_sticker", Arc::new(StickerGeneratorPlugin));
        plugins.insert("get_current_weather", Arc::new(WeatherFallbackPlugin));
        plugins.insert("instantiate_autonomous_task", Arc::new(crate::common::tools::plugins::instantiate_autonomous_task::InstantiateAutonomousTaskPlugin));


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
    

    pub async fn generate_tool_for_prompt(&self, intents: &[String]) -> Tool {
        let mut tools = Vec::new();

        // 🌟 ALWAYS FORCE INJECT DISCOVER_TOOLS NATIVELY
        if let Some(discover_plugin) = self.plugins.get("discover_tools") {
            let mut schema = discover_plugin.schema();
            Self::sanitize_schema_for_gemini(&mut schema);
            if let Ok(func_decl) = serde_json::from_value::<gemini_rust::FunctionDeclaration>(schema) {
                tools.push(func_decl);
            }
        }

        // 🌟 ALWAYS FORCE INJECT SUGGEST_NEW_SKILL IF DISCOVERY IS ACTIVE
        if intents.contains(&"SYSTEM_INTERNAL_DISCOVERY".to_string()) {
            if let Some(suggest_plugin) = self.plugins.get("suggest_new_skill") {
                let mut schema = suggest_plugin.schema();
                Self::sanitize_schema_for_gemini(&mut schema);
                if let Ok(func_decl) = serde_json::from_value::<gemini_rust::FunctionDeclaration>(schema) {
                    tools.push(func_decl);
                }
            }
        }

        // 1. Static Plugable Tools
        for (name, plugin) in &self.plugins {
            if *name == "discover_tools" {
                continue;
            }
            let plugin_intents = plugin.matching_intents();
            let is_matched = intents.iter().any(|i| plugin_intents.contains(&i.as_str()))
                || intents.contains(&"FULL_REGISTRY".to_string())
                || intents.contains(&"HTO_WORKFLOW_REGISTRY".to_string());

            // ⚠️ RECURSION BLACKLIST: Never allow an autonomous workflow task to recursively spawn another autonomous task
            if is_matched && intents.contains(&"HTO_WORKFLOW_REGISTRY".to_string()) && *name == "instantiate_autonomous_task" {
                continue;
            }

            if is_matched {
                let mut schema = plugin.schema();

                // 🌟 SHADOW INJECTION: Fetch runtime optimizations for this static tool handle
                if let Ok(Some(row)) = sqlx::query(
                    "SELECT enriched_description FROM static_plugin_reinforcements WHERE plugin_slug = $1"
                )
                .bind(*name)
                .fetch_optional(&self.pool)
                .await {
                    use sqlx::Row;
                    if let Ok(reinforced_desc) = row.try_get::<String, _>("enriched_description") {
                        if let Some(obj) = schema.as_object_mut() {
                            // Hydrate the compiled description with the dynamically learned variation context!
                            obj.insert("description".to_string(), serde_json::Value::String(reinforced_desc));
                        }
                    }
                }

                Self::sanitize_schema_for_gemini(&mut schema);

                if let Ok(func_decl) = serde_json::from_value::<FunctionDeclaration>(schema) {
                    tools.push(func_decl);
                }
            }
        }

        // 2. Dynamic Edge Plugins
        #[derive(sqlx::FromRow)]
        struct EdgeFnRow {
            slug: String,
            description: String,
            schema_json: serde_json::Value,
        }

        let dynamic_plugins = if intents.contains(&"FULL_REGISTRY".to_string()) || intents.contains(&"HTO_WORKFLOW_REGISTRY".to_string()) {
            sqlx::query_as::<_, EdgeFnRow>(
                "SELECT slug, description, schema_json FROM edge_functions"
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, EdgeFnRow>(
                "SELECT slug, description, schema_json FROM edge_functions WHERE intents && $1"
            )
            .bind(intents)
            .fetch_all(&self.pool)
            .await
        };

        if let Ok(plugins) = dynamic_plugins {
            for p in plugins {
                let mut schema = p.schema_json.clone();

                // 🌟 SHADOW INJECTION: Fetch runtime optimizations for this dynamic tool handle
                if let Ok(Some(row)) = sqlx::query(
                    "SELECT enriched_description FROM static_plugin_reinforcements WHERE plugin_slug = $1"
                )
                .bind(&p.slug)
                .fetch_optional(&self.pool)
                .await {
                    use sqlx::Row;
                    if let Ok(reinforced_desc) = row.try_get::<String, _>("enriched_description") {
                        if let Some(obj) = schema.as_object_mut() {
                            // Hydrate the dynamic description with the learned variation context!
                            obj.insert("description".to_string(), serde_json::Value::String(reinforced_desc));
                        }
                    }
                }

                // Ensure name and description match the DB record handle if not already set by reinforcement
                if let Some(obj) = schema.as_object_mut() {
                    if !obj.contains_key("name") {
                        obj.insert("name".to_string(), serde_json::Value::String(p.slug.clone()));
                    }
                    if !obj.contains_key("description") {
                        obj.insert("description".to_string(), serde_json::Value::String(p.description.clone()));
                    }
                }
                
                Self::sanitize_schema_for_gemini(&mut schema);

                if let Ok(func_decl) = serde_json::from_value::<FunctionDeclaration>(schema) {
                    tools.push(func_decl);
                }
            }
        }

        // De-duplicate tools based on name
        let mut unique_tools = Vec::new();
        let mut seen_names = std::collections::HashSet::new();
        for t in tools {
            if seen_names.insert(t.name.clone()) {
                unique_tools.push(t);
            }
        }

        Tool::with_functions(unique_tools)
    }

    fn sanitize_schema_for_gemini(value: &mut serde_json::Value) {
        if let Some(obj) = value.as_object_mut() {
            obj.remove("additionalProperties");
            for (_, v) in obj.iter_mut() {
                Self::sanitize_schema_for_gemini(v);
            }
        } else if let Some(arr) = value.as_array_mut() {
            for v in arr.iter_mut() {
                Self::sanitize_schema_for_gemini(v);
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
