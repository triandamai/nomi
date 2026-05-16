pub mod agent_model;
pub mod classification;

use crate::common::agent::agent_model::{ChatResponse, PromptActor};
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::common::tools::tools_model::ToolResult;
use crate::common::tools::{NomiTool, ToolDispatcher};
use crate::feature::conversation::model::ChatStreamChunk;
use crate::prompts::PromptRegistry;
use chrono::Utc;
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, GenerationResponse, Message, Role, UsageMetadata,
    Tool,
};
use std::sync::Arc;
use tokio_stream::StreamExt;
use tracing::{error, info};

pub async fn send_prompt(
    dispatcher: &ToolDispatcher,
    actor: PromptActor,
    intents: &[String],
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    info!("\n ==== sending message to llm ==== \n");

    let gemini = dispatcher.gemini.as_ref();

    let gemini_builder = match actor {
        PromptActor::User {
            history,
            memories,
            message,
            system_prompt,
            media,
        } => {
            info!("\n ==== sending user prompt ===== \n");
            let build_prompt = build_system_prompt(history, memories, system_prompt);
            // info!("system prompt\n ${}\n ========",build_prompt);
            let mut user_parts = vec![gemini_rust::Part::Text {
                text: message,
                thought: None,
                thought_signature: None,
            }];

            if let Some((mime_type, data)) = media {
                user_parts.push(gemini_rust::Part::InlineData {
                    inline_data: gemini_rust::Blob { mime_type, data },
                    media_resolution: None,
                });
            }

            let mut builder = gemini
                .generate_content()
                .with_system_prompt(build_prompt)
                .with_message(Message {
                    role: Role::User,
                    content: Content {
                        parts: Some(user_parts),
                        role: Some(Role::User),
                    },
                });

            let tool = if !intents.contains(&"GENERAL".to_string()) || intents.len() > 1 {
                dispatcher.generate_tool_for_prompt(intents)
            } else {
                dispatcher.generate_tool_for_prompt(&["FULL_REGISTRY".to_string()])
            };

            let has_functions = match &tool {
                Tool::Function { function_declarations } => !function_declarations.is_empty(),
                _ => true, // Other tools like GoogleSearch are not empty functions
            };

            if has_functions {
                builder = builder
                    .with_tool(tool)
                    .with_function_calling_mode(FunctionCallingMode::Auto);
            }

            builder.with_max_output_tokens(4096)
        }
        PromptActor::MultiTool {
            history,
            memories,
            message,
            system_prompt,
            tool_turns,
            media,
        } => {
            info!("\n ==== sending tool prompt ===== \n");
            let build_prompt = build_system_prompt(history, memories, system_prompt);
            // info!("system prompt\n ${}\n ========",build_prompt);
            let mut user_parts = vec![gemini_rust::Part::Text {
                text: message,
                thought: None,
                thought_signature: None,
            }];

            if let Some((mime_type, data)) = media {
                user_parts.push(gemini_rust::Part::InlineData {
                    inline_data: gemini_rust::Blob { mime_type, data },
                    media_resolution: None,
                });
            }

            let mut builder = gemini
                .generate_content()
                .with_system_prompt(build_prompt)
                .with_message(Message {
                    role: Role::User,
                    content: Content {
                        parts: Some(user_parts),
                        role: Some(Role::User),
                    },
                });

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

            let tool = if !intents.contains(&"GENERAL".to_string()) || intents.len() > 1 {
                dispatcher.generate_tool_for_prompt(intents)
            } else {
                dispatcher.generate_tool_for_prompt(&["FULL_REGISTRY".to_string()])
            };

            let has_functions = match &tool {
                Tool::Function { function_declarations } => !function_declarations.is_empty(),
                _ => true,
            };

            if has_functions {
                builder = builder
                    .with_tool(tool)
                    .with_function_calling_mode(FunctionCallingMode::Auto);
            }

            builder.with_max_output_tokens(4096)
        }
    };

    // Task 4: Increase timeout
    // Assuming gemini_rust supports with_timeout on the builder
    // gemini_builder = gemini_builder.with_timeout(Duration::from_secs(120));

    // D. Streaming Egress
    let mut stream = match gemini_builder.execute_stream().await {
        Ok(s) => s,
        Err(e) => {
            error!("Gemini stream start failed: {}", e);
            return Err(e.to_string());
        }
    };

    let mut accumulated_text = String::new();
    let mut last_response: Option<GenerationResponse> = None;
    let mut all_function_calls = Vec::new();

    while let Some(res_result) = stream.next().await {
        match res_result {
            Ok(res) => {
                accumulated_text.push_str(&res.text());
                // Clone the function calls to own them and avoid lifetime issues
                all_function_calls.extend(res.function_calls().into_iter().cloned());
                last_response = Some(res);
            }
            Err(e) => {
                error!("Stream chunk error: {}", e);
                if accumulated_text.trim().ends_with(':') {
                    error!("Stream stopped at a colon! Chunk error: {}", e);
                }
                return Err(e.to_string());
            }
        }
    }

    let Some(mut s) = last_response else {
        return Err("Empty stream response".to_string());
    };

    // Aggregate everything into the final response object
    if let Some(candidate) = s.candidates.first_mut() {
        let content = &mut candidate.content;
        // if let Some(content) = &mut candidate.content {
        let mut new_parts = vec![gemini_rust::Part::Text {
            text: accumulated_text.clone(),
            thought: None,
            thought_signature: None,
        }];

        for call in all_function_calls {
            new_parts.push(gemini_rust::Part::FunctionCall {
                function_call: call.clone(),
                thought_signature: None,
            });
        }
        content.parts = Some(new_parts);
        //}
    }

    let raw_text = accumulated_text;
    info!(
        "\n ===== raw response (streamed) ===== \n {} \n ================ \n",
        raw_text
    );

    // Task 1: Heal tags if broken
    let mut healed_text = crate::common::format::heal_thinking_tags(&raw_text);
    if healed_text != raw_text {
        info!(
            "\n ===== healed response ===== \n {} \n ================ \n",
            healed_text
        );
    }

    let mut parse = parse_llm_output(&healed_text);

    // Task 2: Refiner Utility
    // If thought is empty OR response still looks like it contains 'thinking' at the start
    if parse.thought.is_empty()
        && (parse.response.to_lowercase().starts_with("thinking")
            || parse.response.contains("<thinking>"))
    {
        if let Ok(refined_text) =
            crate::common::format::refine_output(&raw_text, dispatcher.gemini.as_ref()).await
        {
            healed_text = refined_text;
            parse = parse_llm_output(&healed_text);
        }
    }

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
        error: None,
    };
    Ok((s, payload))
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

            // Check if it's a plugin tool
            if let Some(plugin) = dispatcher.plugins.get(call_name.as_str()) {
                let plugin_res = plugin.execute(&dispatcher, args).await;

                let tz: chrono_tz::Tz = "Asia/Jakarta".parse().unwrap_or(chrono_tz::UTC);
                let now_wib = Utc::now().with_timezone(&tz);
                let timestamp = format!("**WIB: {}**", now_wib.format("%Y-%m-%d %H:%M"));

                let result = match plugin_res {
                    Ok(content) => ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: format!("{} \n {}", timestamp, content),
                        follow_up_prompt: "".to_string(),
                    },
                    Err(e) => ToolResult {
                        error: format!("Plugin Execution Error: {}", e),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    },
                };

                if let Some(sse) = sse.as_ref() {
                    let _ = sse
                        .send(SseBuilder::new(
                            SseTarget::broadcast("tool_end".to_string()),
                            serde_json::json!({ "name": call_name, "success": result.success }),
                        ))
                        .await;
                }

                return (call_name, result);
            }

            // Legacy & Dynamic Dispatch
            // Normalize tool names to match the ArtaTool enum variants
            let normalized_name = match call_name.as_str() {
                "update_conversation_soul" => "update_nomi_soul",
                "execute_read_query" => "execute_sql_query",
                "parse_to_json" => "parse_string_to_json",
                _ => call_name.as_str(),
            }
            .to_string();

            let tool_json = serde_json::json!({
                "tool": normalized_name,
                "args": {
                    "params": args,
                    "user_message": user_message
                }
            });

            let result = match serde_json::from_value::<NomiTool>(tool_json) {
                Ok(arta_tool) => dispatcher.dispatch(arta_tool).await,
                Err(e) => ToolResult {
                    error: format!("Failed to parse tool {}: {}", call_name, e),
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

    let re_thinking = regex::Regex::new(r"(?si)<thinking>(.*?)</thinking>").unwrap();
    let re_unclosed = regex::Regex::new(r"(?si)<thinking>(.*)").unwrap();

    if let Some(caps) = re_thinking.captures(raw_text) {
        thought = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        clean_content = re_thinking.replace(raw_text, "").to_string();
    } else if let Some(caps) = re_unclosed.captures(raw_text) {
        thought = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        clean_content = re_unclosed.replace(raw_text, "").to_string();
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
