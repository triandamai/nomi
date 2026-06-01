use crate::AppState;
use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;

#[derive(Clone)]
pub struct RagRetrieval {
    pub state: AppState,
    pub dispatcher: ToolDispatcher,
    pub history_limit: Option<usize>,
    pub retrieval_query: Option<String>,
    pub simple_history: bool,
    pub message: String,
    pub system_prompt: String,
    pub media: Option<(String, String)>,
    pub tool_turns: Vec<(Vec<gemini_rust::FunctionCall>, Vec<(String, ToolResult)>)>,
    pub intents: Vec<String>,
    pub history_override: Option<String>,
    pub memories_override: Option<String>,
    pub pending_media_cache: std::sync::Arc<tokio::sync::OnceCell<Option<(String, String)>>>,
}

impl RagRetrieval {
    pub fn new(state: AppState, dispatcher: ToolDispatcher) -> Self {
        Self {
            state,
            dispatcher,
            history_limit: None,
            retrieval_query: None,
            simple_history: false,
            message: String::new(),
            system_prompt: String::new(),
            media: None,
            tool_turns: Vec::new(),
            intents: Vec::new(),
            history_override: None,
            memories_override: None,
            pending_media_cache: std::sync::Arc::new(tokio::sync::OnceCell::new()),
        }
    }

    pub fn with_history(mut self, limit: usize) -> Self {
        self.history_limit = Some(limit);
        self
    }

    pub fn with_retrieval(mut self, query: String) -> Self {
        self.retrieval_query = Some(query);
        self
    }

    pub fn with_simple_history(mut self, simple: bool) -> Self {
        self.simple_history = simple;
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn with_system_prompt(mut self, prompt: String) -> Self {
        self.system_prompt = prompt;
        self
    }

    pub fn with_media(mut self, media: Option<(String, String)>) -> Self {
        self.media = media;
        self
    }

    pub fn with_tool_turns(mut self, turns: Vec<(Vec<gemini_rust::FunctionCall>, Vec<(String, ToolResult)>)>) -> Self {
        self.tool_turns = turns;
        self
    }

    pub fn with_intents(mut self, intents: Vec<String>) -> Self {
        self.intents = intents;
        self
    }

    pub fn with_history_override(mut self, history: String) -> Self {
        self.history_override = Some(history);
        self
    }

    pub fn with_memories_override(mut self, memories: String) -> Self {
        self.memories_override = Some(memories);
        self
    }

    pub fn append_history(mut self, text: &str) -> Self {
        if let Some(ref mut hist) = self.history_override {
            hist.push_str(text);
        } else {
            self.history_override = Some(text.to_string());
        }
        self
    }

    /// Resolves both history formatting and RAG memory queries asynchronously.
    pub async fn execute(&self) -> anyhow::Result<(String, String)> {
        let history = self.fetch_history().await?;
        let memories = self.fetch_memories().await?;
        Ok((history, memories))
    }

    pub async fn fetch_history(&self) -> anyhow::Result<String> {
        if let Some(ref hist) = self.history_override {
            return Ok(hist.clone());
        }

        let Some(conversation_id) = self.dispatcher.conversation_id else {
            return Ok(String::new());
        };
        let limit = self.history_limit.unwrap_or(15) as i64;

        if self.simple_history {
            #[derive(sqlx::FromRow)]
            struct SimpleTurn {
                role: String,
                content: String,
            }

            let turns = sqlx::query_as::<_, SimpleTurn>(
                "SELECT role, content FROM ( \
                     SELECT role, content, created_at FROM messages \
                     WHERE conversation_id = $1 \
                     ORDER BY created_at DESC LIMIT $2 \
                 ) sub ORDER BY created_at ASC"
            )
            .bind(conversation_id)
            .bind(limit)
            .fetch_all(&self.dispatcher.pool)
            .await?;

            let history_text = turns
                .into_iter()
                .map(|m| format!("[{}]: {}", m.role, m.content))
                .collect::<Vec<String>>()
                .join("\n");
            Ok(history_text)
        } else {
            let rows = sqlx::query(
                "SELECT 
                    messages.id,
                    messages.created_at,
                    messages.role,
                    messages.content,
                    messages.thought,
                    messages.user_id,
                    messages.total_tokens,
                    messages.reply_to_id,
                    users.display_name as display_name,
                    messages.image_url,
                    messages.video_url,
                    messages.audio_url,
                    messages.document_url,
                    messages.sticker_url,
                    messages.metadata,
                    CASE WHEN messages.reply_to_id IS NOT NULL THEN
                     jsonb_build_object(
                       'id', rm.id,
                       'role', rm.role,
                       'content', rm.content,
                       'display_name', ru.display_name
                     )
                    ELSE NULL END as replied_message
                FROM messages 
                LEFT JOIN users ON users.id = messages.user_id
                LEFT JOIN messages AS rm ON rm.id = messages.reply_to_id
                LEFT JOIN users AS ru ON ru.id = rm.user_id
                WHERE messages.conversation_id = $1
                ORDER BY created_at DESC LIMIT $2"
            )
            .bind(conversation_id)
            .bind(limit)
            .fetch_all(&self.dispatcher.pool)
            .await?;

            use sqlx::Row;
            let history: Vec<crate::common::repository::message_repo::MessageItemWithDisplay> = rows.into_iter().map(|r| {
                crate::common::repository::message_repo::MessageItemWithDisplay {
                    id: r.get("id"),
                    conversation_id,
                    created_at: r.get("created_at"),
                    role: r.get("role"),
                    content: r.get("content"),
                    thought: r.get("thought"),
                    display_name: r.get("display_name"),
                    user_id: r.get("user_id"),
                    total_tokens: r.get("total_tokens"),
                    image_url: r.get("image_url"),
                    video_url: r.get("video_url"),
                    audio_url: r.get("audio_url"),
                    document_url: r.get("document_url"),
                    sticker_url: r.get("sticker_url"),
                    metadata: r.get("metadata"),
                    reply_to_id: r.get("reply_to_id"),
                    replied_message: r.get("replied_message"),
                }
            }).collect();

            let history_text = crate::feature::message_processor::history_utils::HighFidelityHistory::format_messages(
                history,
                &self.dispatcher.storage,
            );

            Ok(history_text)
        }
    }

    pub async fn fetch_memories(&self) -> anyhow::Result<String> {
        if let Some(ref mems) = self.memories_override {
            return Ok(mems.clone());
        }

        let Some(ref query) = self.retrieval_query else {
            return Ok(String::new());
        };
        if query.trim().is_empty() {
            return Ok(String::new());
        }

        let embedding = crate::rag::get_embedding(&self.dispatcher.gemini_api_key, query).await;
        let memories_text = match embedding {
            Ok(emb) => {
                crate::utils::rag::hybrid_retrieve(
                    &self.dispatcher.pool,
                    query,
                    emb.embedding.values,
                    self.dispatcher.conversation_id,
                    None,
                    None,
                )
                .await
                .unwrap_or_default()
                .join("---")
            }
            Err(_) => String::new(),
        };

        Ok(memories_text)
    }

    /// Retrieves (and caches) the raw pending media database record for the active conversation
    pub async fn get_pending_media(&self) -> Option<(String, String)> {
        let conversation_id = self.dispatcher.conversation_id?;
        self.pending_media_cache
            .get_or_init(|| async {
                crate::common::repository::message_repo::get_latest_unprocessed_media(
                    &self.dispatcher.pool,
                    conversation_id,
                )
                .await
                .ok()
                .flatten()
            })
            .await
            .clone()
    }

    /// Fetches the raw media base64 data for any active visual buffer in the conversation
    pub async fn fetch_raw_media(&self) -> Option<(String, String)> {
        let pending_media = self.get_pending_media().await?;

        let (url, _type) = pending_media;
        let base_url = dotenvy::var("PUBLIC_GATEWAY_URL")
            .unwrap_or("http://localhost:8000/api".to_string());
        let file_path = if url.starts_with("http") && url.contains(&base_url) {
            url.replace(&format!("{}/files/", base_url), "")
        } else {
            url.clone()
        };

        if let Ok(data) = self.dispatcher
            .storage
            .get_file("conversations".to_string(), file_path.clone())
            .await
        {
            let mime_type = mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string();

            // Gemini rejects generic octet-stream. Force image fallbacks for multimodal safety.
            let safe_mime = if mime_type == "application/octet-stream" {
                if file_path.to_lowercase().ends_with(".png") {
                    "image/png".to_string()
                } else if file_path.to_lowercase().ends_with(".webp") {
                    "image/webp".to_string()
                } else {
                    "image/jpeg".to_string()
                }
            } else {
                mime_type
            };

            use base64::Engine;
            let base64_data = base64::engine::general_purpose::STANDARD.encode(data.to_vec());
            tracing::info!(
                "Multimodal: Prepared media context (mime: {})",
                safe_mime
            );
            Some((safe_mime, base64_data))
        } else {
            None
        }
    }

    pub async fn generate_system_prompt(
        &self,
        current_user: &Option<crate::common::identity::UserIdentity>,
        conversation: &crate::feature::Conversation,
        intents_val: &[String],
        memories_text: &str,
    ) -> anyhow::Result<String> {
        let mut combined = String::new();

        // 1. Identity Extraction & Clean Name Logic
        let raw_display_name = current_user
            .as_ref()
            .map(|u| u.display_name.clone())
            .unwrap_or_else(|| "Human".to_string());

        let is_technical_id = |s: &str| {
            let s = s.trim();
            s.is_empty()
                || s.contains('@') // JID
                || (s.contains('-') && s.len() > 20) // UUID
                || s.chars().all(|c| c.is_numeric() || c == '+') // Phone/Telegram ID
        };

        let safe_name = if is_technical_id(&raw_display_name) {
            "Human"
        } else {
            &raw_display_name
        };

        // 2. Base Rules with Dynamic Name
        combined.push_str(
            &crate::prompts::PromptRegistry::CORE_RULES.replace("[Human]", safe_name),
        );
        combined.push_str("\n### PROACTIVE BACKGROUND WORKERS\n\
                           - You possess the capability to spawn autonomous background-threaded loops using the `instantiate_autonomous_task` tool to execute multi-step chores, research pipelines, monitoring tasks, or scheduling updates.\n\
                           - When the user asks you to do something complex, multi-step, or requiring background persistence, do NOT wait for explicit permission or ask the user \"Should I start an autonomous task?\". Instead, AUTONOMOUSLY decide to call `instantiate_autonomous_task` immediately to build the plan and kick off the workflow in the background, then casually let the user know you've got it covered and they can watch its progress live in the timeline side-panel!\n");
        combined.push_str(
            &crate::prompts::PromptRegistry::BOUNDARIES.replace("[Human]", safe_name),
        );

        // 3. Dynamic Onboarding & ID Profile
        if let Some(user_identity) = current_user {
            combined.push_str("\n### CURRENT INTERACTOR IDENTITY PROFILE\n");
            combined.push_str(&format!("- Database User ID: {}\n", user_identity.id));

            if is_technical_id(&raw_display_name) {
                combined.push_str("- Verified Speaker Name: UNKNOWN / TECHNICAL ID\n");
                combined.push_str("- Onboarding Protocol: This user has no saved profile name or is using a technical identifier (JID/UUID). DO NOT call them by their ID. Politely ask what they would like you to call them as part of your organic conversation.\n");
            } else {
                combined.push_str(&format!("- Verified Speaker Name: {}\n", raw_display_name));
                combined.push_str(&format!(
                    "- Contextual History Recollections:\n{}\n",
                    memories_text
                ));
            }
        }

        let boot = conversation.bootstrap_content.clone().unwrap_or_default();
        let soul = conversation.soul_content.clone().unwrap_or_default();

        combined.push_str("\n### Identity Layer\n");
        combined.push_str(&boot.replace("[Human]", safe_name));

        // [FIX] Visual Context Injection - Optimized via cached OnceCell
        let pending_media = self.get_pending_media().await;

        if let Some((url, _type)) = pending_media {
            let full_url = self.dispatcher.storage.get_full_url(&url);
            combined.push_str("\n");
            combined.push_str(&format!(
                "### ACTIVE VISUAL BUFFER\n- Current File: {}\n- Instruction: This file is currently 'Active' and ready for tools like `create_sticker` or `log_expense`. Use the URL provided here for the tool call if the user's intent matches.\n",
                full_url
            ));
        }

        if !soul.is_empty() {
            combined.push_str("\n### Current Personality/Soul\n");
            combined.push_str(&soul.replace("[Human]", safe_name));
        }

        // Optimized static timezone constant mapping instead of dynamic parsing
        let timezone_str = "Asia/Jakarta";
        let tz = chrono_tz::Asia::Jakarta;
        use chrono::Utc;
        let now_local = Utc::now().with_timezone(&tz);

        combined.push_str(&format!(
            "\n### Current Contextual Time\n- UTC: {}\n- Local Time: {} ({})\n",
            Utc::now().to_rfc3339(),
            now_local.to_rfc3339(),
            timezone_str
        ));

        combined.push_str("\n### Timezone & Tool Parameter Instructions\n");
        combined.push_str(&format!(
            "The user's current local time is {} (Asia/Jakarta). \n\
             When calling date-range tracking tools like `get_reminder_stats`, you MUST format parameters like `start_after` and `end_before` as absolute strict ISO 8601 strings with offsets.\n\
             For a query about 'today', start_after MUST be formatted exactly as '{}-00:00:00+07:00' and end_before as '{}-23:59:59+07:00'.\n",
            now_local.format("%H:%M"),
            now_local.format("%Y-%m-%d"),
            now_local.format("%Y-%m-%d")
        ));

        combined.push_str("\n### Orchestrator Instructions \n");
        combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());

        if !intents_val.contains(&"GENERAL".to_string()) || intents_val.len() > 1 {
            // Modular Domain Logic from Plugins
            let mut domain_rules = String::new();
            for plugin in self.dispatcher.plugins.values() {
                let plugin_intents = plugin.matching_intents();
                if intents_val
                    .iter()
                    .any(|i| plugin_intents.contains(&i.as_str()))
                    || intents_val.contains(&"FULL_REGISTRY".to_string())
                {
                    let rules = plugin.rules();
                    if !rules.is_empty() && !domain_rules.contains(rules) {
                        domain_rules.push_str(rules);
                    }

                    // 🌟 SHADOW RULE INJECTION: Fetch runtime optimizations for this static tool handle
                    let plugin_slug = plugin.schema()["name"].as_str().unwrap_or_default().to_string();
                    if !plugin_slug.is_empty() {
                        if let Some(row) = sqlx::query("SELECT additional_rules FROM static_plugin_reinforcements WHERE plugin_slug = $1")
                            .bind(&plugin_slug)
                            .fetch_optional(&self.dispatcher.pool)
                            .await
                            .ok()
                            .flatten()
                        {
                            use sqlx::Row;
                            if let Ok(extra_rules) = row.try_get::<Vec<String>, _>("additional_rules") {
                                for rule in extra_rules {
                                    if !rule.is_empty() && !domain_rules.contains(&rule) {
                                        domain_rules.push_str("\n- Learned Operational Guardrail: ");
                                        domain_rules.push_str(&rule);
                                        domain_rules.push_str("\n");
                                    }
                                }
                            }
                        }
                    }
                }
            }
            combined.push_str(&domain_rules);
        }

        if intents_val.contains(&"FULL_REGISTRY".to_string()) {
            combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());
        }

        Ok(combined)
    }

    pub async fn generate_system_task_prompt(
        &self,
        workspace_bootstrap: &Option<String>,
        workspace_soul: &Option<String>,
        trigger: &crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger,
        intents_val: &[String],
    ) -> anyhow::Result<String> {
        let mut combined = String::new();

        let timezone_str = "Asia/Jakarta";
        let tz: chrono_tz::Tz = timezone_str.parse().unwrap_or(chrono_tz::UTC);
        use chrono::Utc;
        let now_local = Utc::now().with_timezone(&tz);

        // 🌟 HIGH-FIDELITY SYSTEM CONTEXT: Standardized tagging for background triggers
        let (trigger_type, trigger_reason) = match trigger {
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::UserRequested { reason } => ("USER_REQUESTED", reason),
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::ProactiveCheck { reason } => ("PROACTIVE_CHECK", reason),
            crate::feature::message_processor::v2_agent_orchestrator::ExecutionTrigger::SystemAlert { reason } => ("SYSTEM_ALERT", reason),
        };

        combined.push_str(&format!(
            "- <SystemContext trigger=\"{}\" reason=\"{}\" timestamp=\"{}\" />\n",
            trigger_type,
            trigger_reason,
            now_local.to_rfc3339()
        ));

        combined.push_str(crate::prompts::PromptRegistry::CORE_RULES);
        combined.push_str(crate::prompts::PromptRegistry::BOUNDARIES);

        let boot = workspace_bootstrap.clone().unwrap_or_default();
        let soul = workspace_soul.clone().unwrap_or_default();

        combined.push_str("### Identity Layer");
        combined.push_str(&boot);
        if !soul.is_empty() {
            combined.push_str("### Current Personality/Soul");
            combined.push_str(&soul);
        }

        combined.push_str(&format!(
            "\
        ### Current Contextual Time\n\
        - UTC: {}\n\
        - Local Time: {} ({})\n",
            Utc::now().to_rfc3339(),
            now_local.to_rfc3339(),
            timezone_str
        ));

        combined.push_str("\n### Timezone Instructions\n");
        combined.push_str(&format!(
            "The user's current local time is {} ({}). When the user asks for a time like \"6:00\", assume they mean this local time and calculate the UTC equivalent for storage using the +07:00 offset or by converting from Asia/Jakarta. ALWAYS provide the due_at in ISO 8601 format including the local offset (e.g., {} ) for the tool call, but you can acknowledge the local time in your thoughts.\n",
            now_local.format("%H:%M"),
            timezone_str,
            now_local.format("%Y-%m-%dT%H:%M:%S%z")
        ));

        combined.push_str("\n### Orchestrator Instructions\n");
        combined.push_str(crate::prompts::PromptRegistry::orchestrator_instructions());

        // Modular Domain Logic from Plugins
        let mut domain_rules = String::new();
        for plugin in self.dispatcher.plugins.values() {
            let plugin_intents = plugin.matching_intents();
            if intents_val
                .iter()
                .any(|i| plugin_intents.contains(&i.as_str()))
                || intents_val.contains(&"FULL_REGISTRY".to_string())
            {
                let rules = plugin.rules();
                if !rules.is_empty() && !domain_rules.contains(rules) {
                    domain_rules.push_str(rules);
                }
            }
        }
        combined.push_str(&domain_rules);

        if intents_val.contains(&"FULL_REGISTRY".to_string()) {
            combined.push_str(crate::prompts::PromptRegistry::tool_usage_guidelines());
        }

        Ok(combined)
    }
}
