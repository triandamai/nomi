use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::{build_follow_up_prompt, ToolDispatcher};
use crate::common::tools::tools_model::ToolResult;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use tracing::info;

pub struct CommunicationPlugin;

impl NomiToolPlugin for CommunicationPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "send_message",
            "description": "Send a message to another user. If you don't have the user's ID, use manage_user tool with action 'search' first. If multiple users are found, ask the user for clarification.",
            "parameters": {
                "type": "object",
                "properties": {
                    "recipient_id": {
                        "type": "string",
                        "description": "The recipient's User ID (UUID)."
                    },
                    "content": {
                        "type": "string",
                        "description": "The message content to send."
                    },
                    "user_message": {
                        "type": "string",
                        "description": "The original user message to provide context."
                    }
                },
                "required": ["recipient_id", "content", "user_message"]
            }
        })
    }

    fn matching_intents(&self) -> &[&str] {
        &["COMMUNICATION"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let recipient_id = args["recipient_id"].as_str().unwrap_or_default();
            let content = args["content"].as_str().unwrap_or_default();
            let user_message = args["user_message"].as_str().unwrap_or_default();

            info!("Sending message to: {}", recipient_id);

            let recipient_uuid = match uuid::Uuid::parse_str(recipient_id) {
                Ok(id) => id,
                Err(_) => {
                    let result = ToolResult {
                        error: "Invalid recipient ID format. Must be a valid User UUID.".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    };
                    return Ok(serde_json::to_string(&result)?);
                }
            };

            // We need to find a channel for the recipient to know where to send it.
            // For simplicity, we'll pick the most recent channel for that user.
            let channel_info = sqlx::query!(
                "SELECT c.channel_type, c.external_id, c.external_chat_id, c.conversation_id FROM channels c JOIN users u ON u.id = c.user_id WHERE c.user_id = $1 ORDER BY c.created_at DESC LIMIT 1",
                recipient_uuid
            ).fetch_optional(&dispatcher.pool).await?;

            match channel_info {
                Some(channel) => {
                    let convo_id = match channel.conversation_id {
                        Some(id) => id,
                        None => {
                            let result = ToolResult {
                                error: "Recipient channel is not linked to a conversation.".to_string(),
                                success: false,
                                content: "".to_string(),
                                follow_up_prompt: "".to_string(),
                            };
                            return Ok(serde_json::to_string(&result)?);
                        }
                    };
                    let sender_uuid = dispatcher.user_id.unwrap_or_default();
                    let message_id = uuid::Uuid::new_v4();

                    // Save the message first to db
                    let _ = sqlx::query!(
                        "INSERT INTO messages (id, conversation_id, user_id, role, content) VALUES ($1, $2, $3, $4, $5)",
                        message_id, convo_id, sender_uuid, "assistant", content
                    ).execute(&dispatcher.pool).await;

                    let message_item = crate::feature::conversation::model::MessageItem {
                        id: message_id,
                        conversation_id: convo_id,
                        role: "assistant".to_string(),
                        content: content.to_string(),
                        total_tokens: Some(0),
                        answer_tokens: Some(0),
                        prompt_tokens: Some(0),
                        thought: None,
                        image_url: None,
                        video_url: None,
                        audio_url: None,
                        document_url: None,
                        sticker_url: None,
                        user_id: Some(sender_uuid),
                        created_at: chrono::Utc::now(),
                    };

                    crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber(
                        &dispatcher.app_state,
                        vec![recipient_uuid],
                        convo_id,
                        crate::feature::MessageSource::Other { name: channel.channel_type.clone() },
                        message_item.to_sse_json(0),
                        message_item
                    ).await;

                    let res_content = format!(
                        "Message sent to {}: {}",
                        recipient_id, content
                    );
                    let result = ToolResult {
                        error: "".to_string(),
                        success: true,
                        content: res_content.clone(),
                        follow_up_prompt: build_follow_up_prompt(
                            user_message.to_string(),
                            res_content,
                            "send_message".to_string(),
                        ),
                    };
                    Ok(serde_json::to_string(&result)?)
                }
                None => {
                    let res_content = format!("No active channel found for user {}", recipient_id);
                    let result = ToolResult {
                        error: res_content.clone(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "".to_string(),
                    };
                    Ok(serde_json::to_string(&result)?)
                }
            }
        }
        .boxed()
    }
}
