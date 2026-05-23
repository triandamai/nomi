use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::{ToolResult, RetrieveKnowledgeParameters};
use crate::common::tools::ToolDispatcher;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::FunctionDeclaration;
use serde_json::Value;
use tracing::info;

pub struct RetrieveKnowledgePlugin;

impl NomiToolPlugin for RetrieveKnowledgePlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "retrieve_knowledge",
                "Search your long-term memory for specific facts, preferences, and project details. Use start_date and end_date (ISO 8601) if the query implies a timeframe (e.g., 'last week', 'yesterday', 'in March'). If general, leave them null.",
                None,
            )
            .with_parameters::<RetrieveKnowledgeParameters>()
        ).unwrap()
    }

    fn rules(&self) -> &str {
        "### KNOWLEDGE RULES \n- Use `update_knowledge_base` to commit to memory. Use `retrieve_knowledge` to recall facts.\n"
    }

    fn matching_intents(&self) -> &[&str] {
        &["RETRIEVE_KNOWLEDGE", "SEARCH_MEMORIES", "ASK_KNOWLEDGE_BASE", "FIND_FACTS", "STORAGE", "DASHBOARD"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let params: RetrieveKnowledgeParameters = serde_json::from_value(args)?;
            info!("Retrieving knowledge via plugin for query: {}", params.query);

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
            let embedding_res = crate::rag::get_embedding(&dispatcher.gemini_api_key, &params.query).await;

            match embedding_res {
                Ok(embedding) => {
                    let results = crate::utils::rag::hybrid_retrieve(
                        &dispatcher.pool,
                        &params.query,
                        embedding.embedding.values,
                        dispatcher.conversation_id,
                        start_date,
                        end_date,
                    )
                    .await;

                    match results {
                        Ok(memories) => {
                            let content = memories.join("\n---\n");
                            Ok(ToolResult {
                                error: "".to_string(),
                                success: true,
                                content,
                                follow_up_prompt: "".to_string(),
                                ref_id: "".to_string(),
                            })
                        }
                        Err(e) => Ok(ToolResult {
                            error: format!("Error retrieving knowledge: {}", e),
                            success: false,
                            content: "".to_string(),
                            follow_up_prompt: "".to_string(),
                            ref_id: "".to_string(),
                        }),
                    }
                }
                Err(e) => Ok(ToolResult {
                    error: format!("Error generating embedding: {}", e),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                    ref_id: "".to_string(),
                }),
            }
        }
        .boxed()
    }
}
