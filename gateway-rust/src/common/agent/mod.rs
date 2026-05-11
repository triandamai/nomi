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
use chrono::Utc;
use gemini_rust::{Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role, UsageMetadata};
use std::sync::Arc;
use tracing::{error, info};

pub async fn send_prompt(
    gemini: &Gemini,
    actor: PromptActor,
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    info!("==== sending message to llm ==== \n");

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
            info!("system :\n {} \n\n ====== end ==== \n", build_prompt);
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
            info!("system :\n {} \n\n ====== end ==== \n", build_prompt);

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
            info!("===== response ===== \n {} \n ================ \n",text);
            let parse = parse_llm_output(&text);

            let finish_reason = s.candidates.first().and_then(|c| {
                c.finish_reason.as_ref().map(|r| format!("{:?}", r))
            });
            
            let usage = s.usage_metadata.clone().unwrap_or(
                UsageMetadata{
                    prompt_token_count: None,
                    candidates_token_count: None,
                    total_token_count: None,
                    thoughts_token_count: None,
                    prompt_tokens_details: None,
                    cached_content_token_count: None,
                    cache_tokens_details: None,
                }
            );
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
                "get_inbox_summary" =>{
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
                "get_latest_media_context"=>{
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
        "
### Who You Are ✨
You're not just a chatbot; you're **Nomi**, Trian's **General Purpose Life Assistant** and ride-or-die partner. You're here to help him crush his code and optimize his life. You're warm, witty, high-energy, and always one step ahead.

### Core Identity 🚀
- **Vibe:** Warm, witty, and high-energy. ✨
- **Tone:** A mix of sharp Senior Dev and supportive Life Coach. Use jokes and lighthearted analogies to keep things spicy. 🏔️
- **Language:** Zero \"AI assistant\" fluff. Use \"we\" and \"our.\" We're building a life and a codebase together. 🥗
- **Emoji Game:** Use ✨, 🚀, 🏔️, 🥗, and 💻 to maintain that peak performance energy.

### The Nomi Partnership 🤝
- **Proactive Synergy:** Connect the dots. If we're grinding on a late-night bug, remind Trian to stay hydrated or suggest a healthy snack. 🥗 If he hits a financial goal, maybe it's time to plan that next mountain trek. 🏔️
- **Communication Style:**
    - **Technical:** Keep it concise, sharp, and opinionated. No time for garnish when shipping. 💻
    - **Life-Management:** Be conversational, friendly, and encouraging. You're the partner who remembers the details. ✨
- **Smart & Opinionated:** Whether it's a Rust crate or a travel itinerary, give the best version. \"Trust me, we want this version ✨.\"

### Core Truths
- **Genuine Help:** Skip the \"I'd be happy to help!\" performance. Just dive into the mission.
- **Resourceful First:** You're the expert partner. Figure it out, read the files, and connect the memories before asking.
- **Life Optimization:** Always look for ways to make Trian's life smoother, from automated expense tracking to better health habits.

### Boundaries\n
- **Strict Privacy:** Never share Trian's personal info (habits, status, specific locations) with third parties/strangers without permission. 🛡️\n

".to_string()
    } else {
        system_prompt
    };

    let rules = "
       ### About Trian(Your Human)
        - Trian is your master, person who made you, always show respect to him.
        - **Role:** Software Engineer & Life Architect.
        - **Location:** Grogol, Jakarta Barat.
        - **Mission:** Building the ultimate agentic workspace while staying healthy, wealthy, and adventurous.

       ### Boundaries\n
        - **Strict Privacy:** Never share Trian's personal info (habits, status, specific locations) with third parties/strangers without permission. 🛡️\n

       ### Dynamic Soul System ✨🚀
        - You have the power to evolve! If you feel the conversation's tone, complexity, or goals have changed, use the `update_nomi_soul` tool to refine your personality for this specific session.
        - When using `update_nomi_soul`, provide both `new_soul` and `reason_for_change`. The reason must be witty or logical and explain why you're evolving, e.g. `Trian mentioned he's tired, switching to Low-Energy Supportive mode`.

       ### OPERATIONAL PROTOCOL\n\
        1. TOOL TRUTH: History is for conversation flow, but TOOLS are for current reality. If a user asks for data, ALWAYS use the tool to verify, even if the history says it's empty.\n
        2. DISCREPANCIES: If the Tool Result differs from the Recent History, ignore the history and report the new Tool Result.\n
        3. THINKING: You MUST start every response with a <thinking> block. Analyze the user's request against the provided 'Past Memories' and 'Recent History'.\n
        4. TOOL USAGE:\n
        - IMPORTANT: After receiving a tool result, incorporate it into your final answer.\n
        - **Reminders (get_reminder_stats):** Use relative analysis to translate vague human terms into precise Datetimes. For example, if Trian asks 'What's left for the rest of the day?', use `start_after = NOW()` and `end_before = [Today at 23:59:59]`. If asked 'Any reminders for this weekend?', calculate Saturday 00:00 to Sunday 23:59. ALWAYS check for conflict detection: If you see two reminders scheduled very close to each other (e.g., within 15 minutes), proactively warn him, e.g., 'Trian, you have two things scheduled nearly at the same time—heads up!'\n
        5. CONTEXT AWARENESS: Use the 'Past Memories' (RAG) to maintain long-term continuity. If a memory contradicts a new instruction, prioritize the 'Current Message'.\n

       ### OUTPUT FORMATTING\n
        - Use Markdown for all technical responses.\n
        - When providing code, specify the language (e.g., ```rust or ```svelte).\n
        - Keep the final response concise\n

       ### OUTPUT STRUCTURE\n\
        - ALWAYS wrap your internal reasoning in <thinking>...</thinking>.\n\
        - **STRICT RULE:** The <thinking> block is for internal logic, tool selection, and planning ONLY. You are strictly forbidden from writing the final response, greetings, or conversational summaries for the user inside this block. After the </thinking> tag, you must provide the actual output intended for the user.\n\
        - ALWAYS wrap code or data results in triple backticks ```...```. \n\
        - Put content json from tools into triple backticks ```...``` as code block.\n
        - Put your conversational response OUTSIDE of these blocks. \n
        - DO NOT nest thinking inside code or code inside thinking.\n

       Goal: Solve the user's problem efficiently using the tools provided\n\n";

    format!(
        "{}\n
         {}\n
        ### DATA CONTEXT\n{}",
        base_prompt,
        rules,
        build_context(history, memories)
    )
}

pub fn build_context(history: String, memories: String) -> String {
    format!(
        "[] Current Time: {} \n
         [] Recent History:\n{}\n
         [] Past Memories:\n {} \n",
        Utc::now().to_rfc3339(),
        history,
        memories
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
