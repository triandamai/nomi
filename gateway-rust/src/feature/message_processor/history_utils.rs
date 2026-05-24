use crate::common::repository::message_repo::MessageItemWithDisplay;
use crate::common::storage::StorageClient;
use crate::common::tools::tools_model::ToolResult;
use chrono::Utc;

pub struct HighFidelityHistory;

impl HighFidelityHistory {
    /// Formats a list of messages into a structured, high-fidelity technical history block.
    pub fn format_messages(
        messages: Vec<MessageItemWithDisplay>,
        storage: &StorageClient,
    ) -> String {
        let mut history_text = String::from("## CONVERSATION_LOG_v2 ##\n");
        
        for m in messages.into_iter().rev() {
            let display = match m.role.as_str() {
                "user" => m.display_name.clone().unwrap_or_else(|| "User".to_string()),
                "assistant" => "Nomi".to_string(),
                _ => "System".to_string(),
            };

            let timestamp = m.created_at.unwrap_or(Utc::now()).format("%Y-%m-%d %H:%M:%S");
            
            // Header: Entry point for the model's parser
            history_text.push_str(&format!("- <MessageEntry timestamp=\"{}\" id=\"{}\">\n", timestamp, m.id));
            history_text.push_str(&format!("    [Actor]: {}\n", display));
            
            // 🌟 HIGH-FIDELITY QUOTED CONTEXT: Standardized tagging for replies
            // Prioritize native joined replied_message, fallback to metadata.quoted_message
            if let Some(replied) = &m.replied_message {
                let q_id = replied.id.to_string();
                let q_sender = replied.display_name.as_ref()
                    .unwrap_or(&replied.role);
                let q_text = &replied.content;
                
                history_text.push_str(&format!("    [SystemEvent: QUOTED_MESSAGE] id=\"{}\" actor=\"{}\"\n", q_id, q_sender));
                history_text.push_str(&format!("    [QuotedContent]: {}\n", q_text));
            } else if let Some(meta) = &m.metadata {
                if let Some(quoted) = meta.get("quoted_message") {
                    let q_id = quoted.get("message_id").and_then(|v| v.as_str()).unwrap_or("UNKNOWN");
                    let q_sender = quoted.get("display_name")
                        .and_then(|v| v.as_str())
                        .or_else(|| quoted.get("sender_id").and_then(|v| v.as_str()))
                        .unwrap_or("UNKNOWN");
                    let q_text = quoted.get("text").and_then(|v| v.as_str()).unwrap_or("");
                    
                    history_text.push_str(&format!("    [SystemEvent: QUOTED_MESSAGE] id=\"{}\" actor=\"{}\"\n", q_id, q_sender));
                    history_text.push_str(&format!("    [QuotedContent]: {}\n", q_text));
                }
            }

            // Content Body
            history_text.push_str(&format!("    [Content]: {}\n", m.content));

            // Media Assets
            if let Some(path) = m.image_url { history_text.push_str(&format!("    [Media: Image] {}\n", storage.get_full_url(&path))); }
            if let Some(path) = m.video_url { history_text.push_str(&format!("    [Media: Video] {}\n", storage.get_full_url(&path))); }
            if let Some(path) = m.audio_url { history_text.push_str(&format!("    [Media: Audio] {}\n", storage.get_full_url(&path))); }
            if let Some(path) = m.document_url { history_text.push_str(&format!("    [Media: Document] {}\n", storage.get_full_url(&path))); }
            if let Some(path) = m.sticker_url { history_text.push_str(&format!("    [Media: Sticker] {}\n", storage.get_full_url(&path))); }

            // Technical Metadata (Structured Context)
            if let Some(meta) = m.metadata {
                if let Some(slug) = meta["proposal_slug"].as_str() {
                    history_text.push_str(&format!("    [SystemEvent: SKILL_PROPOSAL] slug=\"{}\"\n", slug));
                }
                
                if let Some(obj) = meta.as_object() {
                    for (key, value) in obj {
                        if key != "proposal_slug" && key != "is_processed" && !key.starts_with("last_") {
                            history_text.push_str(&format!("    [Metadata: {}] {}\n", key.to_uppercase(), value));
                        }
                    }
                }
            }
            
            history_text.push_str("  </MessageEntry>\n\n");
        }
        
        history_text
    }

    /// Formats a list of tool results from a live turn into the high-fidelity specialized tag format.
    pub fn format_tool_results(
        tool_results: &Vec<(String, ToolResult)>,
    ) -> String {
        let mut output = String::new();
        
        for (name, result) in tool_results {
            let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
            
            // 🌟 SPECIALIZED TOOL TAGGING: Using <ToolResult> for system operations with ref_id traceability
            output.push_str(&format!("- <ToolResult name=\"{}\" ref_id=\"{}\" timestamp=\"{}\" id=\"LIVE_TURN_{}\">\n", name, result.ref_id, timestamp, name));
            
            let status = if result.success { "SUCCESS" } else { "ERROR" };
            let content = if result.success { &result.content } else { &result.error };
            
            output.push_str(&format!("    [Status]: {}\n", status));
            output.push_str(&format!("    [Output]: {}\n", content));
            
            if !result.follow_up_prompt.is_empty() {
                output.push_str(&format!("    [Directive]: {}\n", result.follow_up_prompt));
            }
            
            output.push_str("  </ToolResult>\n\n");
        }
        
        output
    }
}
