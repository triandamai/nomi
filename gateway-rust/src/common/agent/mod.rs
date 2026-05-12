pub mod agent_model;

use crate::common::agent::agent_model::{ChatResponse, PromptActor};
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::common::tools::tools_model::{
    ExecuteReadQueryParameters, ReadWorkSpaceParameters, SearchWebParameters, ToolResult,
    UpdateConversationSoulParameters,
};
use crate::common::tools::{ArtaTool, ToolDispatcher};
use crate::feature::conversation::chat_model::ChatStreamChunk;
use crate::prompts::PromptRegistry;
use chrono::Utc;
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role,
    UsageMetadata,
};
use std::sync::Arc;
use tracing::{error, info};

pub async fn send_prompt(
    gemini: &Gemini,
    actor: PromptActor,
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    info!("\n ==== sending message to llm ==== \n");

    let gemini_builder = match actor {
        PromptActor::User {
            history,
            memories,
            message,
            system_prompt,
        } => {
            info!("\n ==== sending user prompt ===== \n");
            // info!("history text user:\n {} \n", history);
            // info!("memories:\n {} \n", memories);
            let build_prompt = build_system_prompt(history, memories, system_prompt);
            // info!("system :\n {} \n\n ====== end ==== \n", build_prompt);
            gemini
                .generate_content()
                .with_system_prompt(build_prompt)
                .with_user_message(message)
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::Auto)
                .with_max_output_tokens(4096)
        }
        PromptActor::MultiTool {
            history,
            memories,
            message,
            system_prompt,
            tool_turns,
        } => {
            info!("\n ==== sending tool prompt ===== \n");
            // info!("history text user:\n {} \n", history);
            // info!("memories:\n {} \n", memories);
            let build_prompt = build_system_prompt(history, memories, system_prompt);
            // info!("system :\n {} \n\n ====== end ==== \n", build_prompt);

            let mut builder = gemini
                .generate_content()
                .with_system_prompt(build_prompt)
                .with_user_message(message);

            // Add turns of interactions
            for (calls, results) in tool_turns {
                // First, the model's calls for this turn
                let mut call_parts = Vec::new();
                for call in calls {
                    call_parts.push(gemini_rust::Part::FunctionCall {
                        function_call: call,
                        thought_signature: None,
                    });
                }
                if !call_parts.is_empty() {
                    builder = builder.with_message(Message {
                        content: Content {
                            parts: Some(call_parts),
                            role: Some(Role::Model),
                        },
                        role: Role::Model,
                    });
                }

                // Then, the responses for those calls
                let mut response_parts = Vec::new();
                for (name, result) in results {
                    response_parts.push(gemini_rust::Part::FunctionResponse {
                        function_response: gemini_rust::tools::FunctionResponse::new(
                            name,
                            serde_json::to_value(result).unwrap_or_default(),
                        ),
                    });
                }
                if !response_parts.is_empty() {
                    builder = builder.with_message(Message {
                        content: Content {
                            parts: Some(response_parts),
                            role: Some(Role::User),
                        },
                        role: Role::User,
                    });
                }
            }

            builder
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::Auto)
                .with_max_output_tokens(4096)
        }
    };
    // D. Streaming Egress
    match gemini_builder.execute().await {
        Ok(s) => {
            let text = s.text();
            info!("===== response ===== \n {} \n ================ \n", text);
            let parse = parse_llm_output(&text);

            let finish_reason = s
                .candidates
                .first()
                .and_then(|c| c.finish_reason.as_ref().map(|r| format!("{:?}", r)));

            let usage = s.usage_metadata.clone().unwrap_or(UsageMetadata {
                prompt_token_count: None,
                candidates_token_count: None,
                total_token_count: None,
                thoughts_token_count: None,
                prompt_tokens_details: None,
                cached_content_token_count: None,
                cache_tokens_details: None,
            });
            let prompt_tokens = usage.prompt_token_count.unwrap_or(0);
            let answer_tokens = usage.candidates_token_count.unwrap_or(0);
            let total_tokens = usage.total_token_count.unwrap_or(0);

            let payload = ChatStreamChunk {
                content: parse.response,
                thought: parse.thought,
                code_block: parse.code,
                tool_call: None,
                prompt_tokens,
                answer_tokens,
                total_tokens,
                finish_reason,
            };
            Ok((s, payload))
        }
        Err(e) => {
            error!("Gemini stream failed: {}", e);
            Err(e.to_string().to_string())
        }
    }
}

pub async fn execute_tools(
    dispatcher: &ToolDispatcher,
    function_calls: Vec<FunctionCall>,
    user_message: &str,
    sse: Option<Arc<SseBroadcaster>>,
) -> Vec<(String, ToolResult)> {
    let mut futures = Vec::new();

    for call in function_calls {
        let dispatcher = dispatcher.clone();
        let user_message = user_message.to_string();
        let sse = sse.clone();
        let call_name = call.name.clone();
        let args = call.args.clone();

        futures.push(tokio::spawn(async move {
            info!(
                function_name = call_name,
                args = ?args,
                "executing function call"
            );

            // Send tool_start SSE event
            if let Some(sse) = sse.as_ref() {
                let _ = sse
                    .send(SseBuilder::new(
                        SseTarget::broadcast("tool_start".to_string()),
                        serde_json::json!({ "name": call_name }),
                    ))
                    .await;
            }

            let result = match call_name.as_str() {
                "read_workspace_file" => {
                    let param: ReadWorkSpaceParameters = serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::ReadWorkspaceFile {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "execute_read_query" => {
                    let param: ExecuteReadQueryParameters = serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::ExecuteSqlQuery {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "web_search" => {
                    let param: SearchWebParameters = serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::WebSearch {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "read_web_page" => {
                    let param: crate::common::tools::tools_model::ReadWebPageParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::ReadWebPage {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "update_nomi_soul" | "update_conversation_soul" => {
                    let param: UpdateConversationSoulParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::UpdateConversationSoul {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "update_knowledge_base" => {
                    let param: crate::common::tools::tools_model::UpdateKnowledgeBaseParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::UpdateKnowledgeBase {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "evolve_bootstrap_content" => {
                    let param: crate::common::tools::tools_model::EvolveBootstrapParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::EvolveBootstrap {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "create_reminder" => {
                    let param: crate::common::tools::tools_model::CreateReminderParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::CreateReminder {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "modify_reminder" => {
                    let param: crate::common::tools::tools_model::ModifyReminderParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::ModifyReminder {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "get_inbox_summary" => {
                    let param: crate::common::tools::tools_model::GetInboxSummaryParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::GetInboxSummary {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "get_reminder_stats" => {
                    let param: crate::common::tools::tools_model::GetReminderStatsParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::GetReminderStats {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "search_users" => {
                    let param: crate::common::tools::tools_model::SearchUsersParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::SearchUsers {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "update_user_profile" => {
                    let param: crate::common::tools::tools_model::UpdateUserProfileParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::UpdateUserProfile {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "send_direct_message" => {
                    let param: crate::common::tools::tools_model::SendDirectMessageParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::SendDirectMessage {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "make_sticker" => {
                    let param: crate::common::tools::tools_model::MakeStickerParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::MakeSticker {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "log_expense" => {
                    let param: crate::common::tools::tools_model::LogExpenseParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::LogExpense {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "analyze_media" => {
                    let param: crate::common::tools::tools_model::AnalyzeMediaParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::AnalyzeMedia {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                "get_latest_media_context" => {
                    let param: crate::common::tools::tools_model::GetLatestMediaContextParameters =
                        serde_json::from_value(args).unwrap();
                    dispatcher
                        .dispatch(ArtaTool::GetLatestMediaContext {
                            params: param,
                            user_message: user_message.clone(),
                        })
                        .await
                }
                _ => ToolResult {
                    error: format!("Unknown tool: {}", call_name),
                    success: false,
                    content: "".to_string(),
                    follow_up_prompt: "".to_string(),
                },
            };

            // Send tool_end SSE event
            if let Some(sse) = sse.as_ref() {
                let _ = sse
                    .send(SseBuilder::new(
                        SseTarget::broadcast("tool_end".to_string()),
                        serde_json::json!({ "name": call_name, "success": result.success }),
                    ))
                    .await;
            }

            (call_name, result)
        }));
    }

    let results = futures::future::join_all(futures).await;
    results.into_iter().map(|r| r.unwrap()).collect()
}

pub fn build_system_prompt(history: String, memories: String, system_prompt: String) -> String {
    let base_prompt = if system_prompt.trim().is_empty() {
        PromptRegistry::default_soul_prompts().to_string()
    } else {
        system_prompt
    };

    format!(
        "{}\n
         {}\n
        ### DATA CONTEXT\n{}",
        base_prompt,
        PromptRegistry::default_rules_prompts(),
        build_context(history, memories)
    )
}

pub fn build_context(history: String, memories: String) -> String {
    format!(
        "[] Current Time: {} \n\
         [] Past Memories:\n {} \n,
         [] Recent History:\n{}\n",
        Utc::now().to_rfc3339(),
        memories,
        history
    )
}

pub fn parse_llm_output(raw_text: &str) -> ChatResponse {
    // 1. Extract Thinking
    let mut thought = String::new();
    let mut clean_content = raw_text.to_string();

    if let Some(start) = raw_text.find("<thinking>") {
        if let Some(end) = raw_text.find("</thinking>") {
            thought = raw_text[start + 10..end].to_string();
            // Remove the entire <thinking>...</thinking> block from the response
            let before = &raw_text[..start];
            let after = &raw_text[end + 11..];
            clean_content = format!("{}{}", before, after);
        } else {
            // Unclosed thinking tag
            thought = raw_text[start + 10..].to_string();
            clean_content = raw_text[..start].to_string();
        }
    }

    // 2. Extract Code Block (Improved)
    let mut code_block = String::new();
    if let Some(start) = clean_content.find("```") {
        let rest = &clean_content[start + 3..];
        if let Some(end_offset) = rest.find("```") {
            code_block = clean_content[start..start + 3 + end_offset + 3].to_string();
        } else {
            code_block = clean_content[start..].to_string();
        }
    }

    ChatResponse {
        thought: thought.trim().to_string(),
        code: code_block.trim().to_string(),
        response: clean_content.trim().to_string(),
    }
}
