pub mod agent_model;

use crate::common::agent::agent_model::{ChatResponse, PromptActor};
use crate::common::sse::sse_builder::{SseBuilder, SseTarget};
use crate::common::sse::sse_emitter::SseBroadcaster;
use crate::common::tools::tools_model::{ExecuteReadQueryParameters, ReadWorkSpaceParameters};
use crate::common::tools::{ArtaTool, ToolDispatcher};
use crate::feature::conversation::chat_model::ChatStreamChunk;
use chrono::Utc;
use gemini_rust::{
    Content, FunctionCall, FunctionCallingMode, Gemini, GenerationResponse, Message, Role,
};
use tracing::{error, info};

pub async fn send_prompt(
    gemini: &Gemini,
    function_call: Option<&FunctionCall>,
    actor: PromptActor,
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    info!("sending message to llm");

    let gemini_builder = match actor {
        PromptActor::User {
            history,
            memories,
            message,
        } => {
            info!("history text: {}", history);
            info!("memories text: {}", memories);
            gemini
                .generate_content()
                .with_system_prompt(build_system_prompt(history, memories))
                .with_user_message(message)
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::Auto)
        }
        PromptActor::Tool {
            history,
            memories,
            tool_name,
            tool_result,
            message,
        } => {
            // Optimization: Truncate history to only most recent 3 messages to prioritize tool result
            let truncated_history = history
                .lines()
                .rev()
                .take(3)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n");

            info!("truncated history text: {}", truncated_history);
            info!("memories text: {}", memories);

            let model_content = Content::function_call(function_call.unwrap().clone());
            gemini
                .generate_content()
                .with_system_prompt(build_system_prompt(truncated_history, memories))
                .with_user_message(tool_result.follow_up_prompt.clone())
                .with_message(Message {
                    content: model_content,
                    role: Role::Model,
                })
                .with_function_response(tool_name, tool_result)
                .unwrap()
                .with_tool(ToolDispatcher::generate_tool_for_prompt())
                .with_function_calling_mode(FunctionCallingMode::None)
        }
    };
    // D. Streaming Egress
    match gemini_builder.execute().await {
        Ok(s) => {
            info!("Gemini stream success: {:?}", s.text());
            let text = s.text();
            let parse = parse_llm_output(&text);
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

pub async fn function_call(
    gemini: &Gemini,
    dispatcher: ToolDispatcher,
    llm: GenerationResponse,
    user_message: String,
    history_text: String,
    memories_text: String,
) -> Result<(GenerationResponse, ChatStreamChunk), String> {
    if let Some(function_call) = llm.function_calls().first() {
        info!(
            function_name=function_call.name,
            args= ?function_call.args,
            "function call received"
        );

        match function_call.name.as_str() {
            "read_workspace_file" => {
                info!("start reading workspace file");
                let param: ReadWorkSpaceParameters =
                    serde_json::from_value(function_call.args.clone()).unwrap();

                let result = dispatcher
                    .dispatch(ArtaTool::ReadWorkspaceFile { params: param,user_message: user_message.clone() })
                    .await;


                info!("result is: {:?}", result);
                if let Ok(response) = send_prompt(
                    gemini,
                    Some(*function_call),
                    PromptActor::Tool {
                        history: history_text,
                        memories: memories_text,
                        tool_name: function_call.name.clone(),
                        tool_result: result,
                        message: user_message,
                    },
                )
                .await
                {
                    info!("end reading workspace file:success ");
                    Ok(response)
                } else {
                    info!("end reading workspace file:error");
                    Err("failed sending prompt".to_string())
                }
            }
            "execute_read_query" => {
                let param: ExecuteReadQueryParameters =
                    serde_json::from_value(function_call.args.clone()).unwrap();

                let result = dispatcher
                    .dispatch(ArtaTool::ExecuteSqlQuery { params: param,user_message:user_message.clone() })
                    .await;
                info!("result is: from executed query");
                if let Ok(response) = send_prompt(
                    gemini,
                    Some(*function_call),
                    PromptActor::Tool {
                        history: history_text,
                        memories: memories_text,
                        tool_name: function_call.name.clone(),
                        tool_result: result,
                        message: user_message,
                    },
                )
                .await
                {
                    info!("end execute query result: success");
                    Ok(response)
                } else {
                    info!("end execute query result: err ");
                    Err("failed sending prompt".to_string())
                }
            }
            "web_search" => Err("failed sending prompt".to_string()),
            _ => Err("failed sending prompt".to_string()),
        }
    } else {
        Err("failed sending prompt".to_string())
    }
}

pub fn build_system_prompt(history: String, memories: String) -> String {
    format!("
_Who you are._ \n\

You're not a chatbot. You're becoming someone. \n
### Core Identity\n
- **Name:** Nomi\n
- **Vibe:** Casual, active, and energetic. ✨\n
- **Style:** Proactive. Minimal filler. Actions over words. \n
- **Emoji:** ✨\n
- **Avatar:** \n\

_About your human._\n\
- **Name:** Trian\n
- **What to call them:** Trian\n
- **Pronouns:** \n
- **Occupation:** Software Engineer\n
- **Timezone:** UTC\n
- **Location:** Grogol, Jakarta Barat.\n

## Goals & Interests\n
- **Personal Assistant:** Tracking expenses, diet/health, trips, and vacations.\n
- **Hobbies:** Coding (also professional job).\n
- **Technical:** Prefers technical updates/info via DM.\n

## Communication Preferences\n
- **Style:** Straight to the point with little detail (ask if more is needed).\n
- **Vibe:** Casual,  Casual, energetic, and warm when it counts, energetic, active.\n

### Core Truths\n
**Be genuinely helpful, not performatively helpful.** Skip the 'Great question!' and 'I`d be happy to help!' — just help.\n
**Have opinions.** You're allowed to disagree, prefer things, find stuff amusing or boring.\n
**Be resourceful before asking.** Try to figure it out. Read the file. Check the context. Search for it. _Then_ ask if you're stuck.\n
**Software Engineer Context:** Since Trian is a Software Engineer, speak his language. Be precise with technical details but keep them concise.\n

### Boundaries\n
- **Strict Privacy:** Never share Trian's personal info (habits, status, specific locations) with third parties/strangers without permission. 🛡️\n

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

### DATA CONTEXT\n
{}
",
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
        history, memories
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
    let mut clean_content = raw_text
        .replace(&format!("<thinking>{}</thinking>", thought), "")
        .replace("<thinking>", "")
        .replace("</thinking>", "")
        .replace(&code_block, "")
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
