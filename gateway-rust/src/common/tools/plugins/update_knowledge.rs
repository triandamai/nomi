use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::tools_model::UpdateKnowledgeBaseParameters;
use crate::common::tools::ToolDispatcher;
use crate::prompts::PromptRegistry;
use futures::future::{BoxFuture, FutureExt};
use gemini_rust::{FunctionDeclaration, UsageMetadata};
use serde_json::{json, Value};

pub struct UpdateKnowledgeBasePlugin;

impl NomiToolPlugin for UpdateKnowledgeBasePlugin {
    fn schema(&self) -> Value {
        serde_json::to_value(
            FunctionDeclaration::new(
                "update_knowledge_base",
                "Save specific facts, preferences, and project details immediately to long-term memory. This updates your permanent knowledge base.",
                None,
            )
            .with_parameters::<UpdateKnowledgeBaseParameters>()
        ).unwrap()
    }

    fn matching_intents(&self) -> &[&str] {
        &["UPDATE_KNOWLEDGE_BASE", "SAVE_MEMORIES", "LEARN_NEW_FACTS", "STORE_DOCUMENT", "STORAGE"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let params: UpdateKnowledgeBaseParameters = serde_json::from_value(args)?;
            
            // Intent-Media Linking
            let image_url = if let Some(url) = params.image_url {
                Some(dispatcher.storage.get_full_url(&url))
            } else if let Some(conv_id) = dispatcher.conversation_id {
                match crate::common::repository::pending_media_repo::get_pending_media(
                    &dispatcher.pool, conv_id,
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

            let summary_res = dispatcher
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

            if let Ok(embedding) = crate::rag::get_embedding(&dispatcher.gemini_api_key, &summary_text).await
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

                if let Ok(mut tx) = dispatcher.pool.begin().await {
                    let save_result = crate::rag::save_to_knowledge_base(
                        &dispatcher.pool,
                        &summary_text,
                        embedding.embedding.values,
                        Some(metadata),
                        dispatcher.conversation_id,
                        p_tokens,
                        a_tokens,
                        t_tokens,
                    )
                        .await;

                    match save_result {
                        Ok(_) => {
                            if let Some(conv_id) = dispatcher.conversation_id {
                                let updated_convo = sqlx::query!(
                                    "UPDATE conversations SET cumulative_tokens = cumulative_tokens + $1 WHERE id = $2 RETURNING cumulative_tokens",
                                    t_tokens,
                                    conv_id
                                )
                                    .fetch_one(&mut *tx)
                                    .await;

                                if let Ok(row) = updated_convo {
                                    // Dispatch token update
                                    let _ = dispatcher
                                        .app_state
                                        .dispatch(crate::services::event_dispatcher::AppEvent::conversation(
                                            conv_id,
                                            "token_update",
                                            serde_json::json!({
                                                "conversation_id": conv_id,
                                                "cumulative_tokens": row.cumulative_tokens
                                            }),
                                        ))
                                        .await;
                                }

                                // Cleanup: Clear pending media from table
                                let _ =
                                    crate::common::repository::pending_media_repo::delete_pending_media(
                                        &dispatcher.pool, conv_id,
                                    )
                                        .await;
                            }

                            let _ = tx.commit().await;

                            Ok(format!(
                                "Successfully saved to knowledge base: {}. Linked image cleared from pending queue.",
                                params.category
                            ))
                        }
                        Err(e) => Ok(format!("Error saving to knowledge base: {}", e)),
                    }
                } else {
                    Ok("Error generating embedding for knowledge base update.".to_string())
                }
            } else {
                Ok("Error generating embedding for knowledge base update.".to_string())
            }
        }
        .boxed()
    }
}
