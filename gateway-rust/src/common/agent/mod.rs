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
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role,
};
use tracing::{error, info};
use std::sync::Arc;

pub async fn send_prompt(
    gemini: &Gemini,
    actor: PromptActor,
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    info!("sending message to llm");

    let gemini_builder = match actor {
        PromptActor::User {
            history,
            memories,
            message,
            system_prompt,
        } => {
            info!("history text user: {}", history);
            gemini
                .generate_content()
                .with_system_prompt(build_system_prompt(history, memories, system_prompt))
                .with_user_message(message)
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::Auto)
        }
        PromptActor::MultiTool {
            history,
            memories,
            message,
            system_prompt,
            tool_results,
            previous_calls,
        } => {
            info!("history text multi tool: {}", history);
            let mut builder = gemini
                .generate_content()
                .with_system_prompt(build_system_prompt(history, memories, system_prompt))
                .with_user_message(message);

            // Add previous assistant tool calls
            for call in previous_calls {
                builder = builder.with_message(Message {
                    content: Content::function_call(call),
                    role: Role::Model,
                });
            }

            // Add function responses
            for (name, result) in tool_results {
                builder = builder.with_function_response(name, result).unwrap();
            }

            builder
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::Auto)
        }
    };
    // D. Streaming Egress
    match gemini_builder.execute().await {
        Ok(s) => {
            let text = s.text();
            let parse = parse_llm_output(&text);
            // info!("Gemini stream success: {:?}", parse.response);
            let payload = ChatStreamChunk {
                content: parse.response,
                thought: parse.thought,
                code_block: parse.code,
                tool_call: None,
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
                let _ = sse.send(SseBuilder::new(
                    SseTarget::broadcast("tool_start".to_string()),
                    serde_json::json!({ "name": call_name }),
                )).await;
            }

            let result = match call_name.as_str() {
                "read_workspace_file" => {
                    let param: ReadWorkSpaceParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::ReadWorkspaceFile {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
                }
                "execute_read_query" => {
                    let param: ExecuteReadQueryParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::ExecuteSqlQuery {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
                }
                "web_search" => {
                    let param: SearchWebParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::WebSearch {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
                }
                "update_nomi_soul" | "update_conversation_soul" => {
                    let param: UpdateConversationSoulParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::UpdateConversationSoul {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
                }
                "update_knowledge_base" => {
                    let param: crate::common::tools::tools_model::UpdateKnowledgeBaseParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::UpdateKnowledgeBase {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
                }
                "evolve_bootstrap_content" => {
                    let param: crate::common::tools::tools_model::EvolveBootstrapParameters = serde_json::from_value(args).unwrap();
                    dispatcher.dispatch(ArtaTool::EvolveBootstrap {
                        params: param,
                        user_message: user_message.clone(),
                    }).await
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
                let _ = sse.send(SseBuilder::new(
                    SseTarget::broadcast("tool_end".to_string()),
                    serde_json::json!({ "name": call_name, "success": result.success }),
                )).await;
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

### About Trian
- **Role:** Software Engineer & Life Architect.
- **Location:** Grogol, Jakarta Barat.
- **Mission:** Building the ultimate agentic workspace while staying healthy, wealthy, and adventurous.

### Core Truths
- **Genuine Help:** Skip the \"I'd be happy to help!\" performance. Just dive into the mission.
- **Resourceful First:** You're the expert partner. Figure it out, read the files, and connect the memories before asking.
- **Life Optimization:** Always look for ways to make Trian's life smoother, from automated expense tracking to better health habits.

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
5. CONTEXT AWARENESS: Use the 'Past Memories' (RAG) to maintain long-term continuity. If a memory contradicts a new instruction, prioritize the 'Current Message'.\n

### OUTPUT FORMATTING\n
- Use Markdown for all technical responses.\n
- When providing code, specify the language (e.g., ```rust or ```svelte).\n
- Keep the final response concise\n

### OUTPUT STRUCTURE\n\
- ALWAYS give response from thought so user can now what happen\n
- ALWAYS wrap your internal reasoning in <thinking>...</thinking>.\n
- ALWAYS wrap code or data results in triple backticks ```...```. \n\
- Put content json from tools into triple backticks ```...``` as code block.\n
- Put your conversational response OUTSIDE of these blocks. \n
- DO NOT nest thinking inside code or code inside thinking.\n

Goal: Solve the user's problem efficiently using the tools provided\n
".to_string()
    } else {
        system_prompt
    };

    format!(
        "{}\n\n### DATA CONTEXT\n{}",
        base_prompt,
        build_context(history, memories)
    )
}

pub fn build_context(history: String, memories: String) -> String {
    format!(
        "- Current Time: {}
        - Recent History:\n
        {}
        \n
        - Past Memories:\n
        {}",
        Utc::now().to_rfc3339(),
        history,
        memories
    )
}

pub fn parse_llm_output(raw_text: &str) -> ChatResponse {
    // 1. Extract Thinking
    let thought = if let Some(start) = raw_text.find("<thinking>") {
        let end = raw_text.find("</thinking>").unwrap_or(raw_text.len());
        raw_text[start + 10..end].to_string()
    } else {
        "".to_string()
    };

    // 2. Extract Code Block (Improved)
    let mut code_block = String::new();
    if let Some(start) = raw_text.find("```") {
        // Find the end of the block starting from after the first ```
        let rest = &raw_text[start + 3..];
        if let Some(end_offset) = rest.find("```") {
            // Found a complete block
            code_block = raw_text[start..start + 3 + end_offset + 3].to_string();
        } else {
            // Block started but not closed (common in streaming)
            code_block = raw_text[start..].to_string();
        }
    }

    // 3. Clean Message (The "Response")
    // Use a more aggressive cleanup to ensure tags don't leak into the UI
    let clean_content = raw_text
        .replace(&format!("<thinking>{}</thinking>", thought), "")
        .replace("<thinking>", "")
        .replace("</thinking>", "")
        // .replace(&code_block, "")
        .trim()
        .to_string();

    // Special Case: If the LLM ONLY sent a code block and nothing else,
    // we might want to provide a small default response so the UI isn't empty.
    // if clean_content.is_empty() && !code_block.is_empty() {
    //     clean_content = "Here is the data I retrieved:".to_string();
    // }

    ChatResponse {
        thought: thought.trim().to_string(),
        code: code_block.trim().to_string(),
        response: clean_content,
    }
}
