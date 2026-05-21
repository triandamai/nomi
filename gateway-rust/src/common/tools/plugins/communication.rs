use crate::common::tools::plugin_trait::NomiToolPlugin;
use crate::common::tools::ToolDispatcher;
use crate::common::tools::tools_model::ToolResult;
use futures::future::{BoxFuture, FutureExt};
use serde_json::{json, Value};
use tracing::info;

pub struct CommunicationPlugin;

impl NomiToolPlugin for CommunicationPlugin {
    fn schema(&self) -> Value {
        json!({
            "name": "send_message",
            "description": "Fires an outbound message payload to a specified target. Accepts a database UUID string, a standard phone JID, or a protocol LID string as the target parameter.",
            "parameters": {
                "type": "object",
                "properties": {
                    "target": {
                        "type": "string",
                        "description": "The target identifier. Can be a raw database UUID string, an @s.whatsapp.net string, or an @lid string."
                    },
                    "message_body": {
                        "type": "string",
                        "description": "The exact textual content of the outbound message to transmit."
                    }
                },
                "required": ["target", "message_body"]
            }
        })
    }

    fn rules(&self) -> &str {
        "\n##OUTBOUND MESSAGING CORE RULES:\n\
        1. When instructed to send a message to a user, always call your user lookup/search tool first.\n\
        2. As soon as the search tool returns an identifier (whether it is a database UUID string, JID, or LID), pass that identifier directly into the 'target' field of this tool.\n\
        3. Do not halt execution or request additional format conversions if the target identifier looks like a UUID or a non-phone string; the native tool wrapper has full responsibility for resolving routing addresses."
    }

    fn matching_intents(&self) -> &[&str] {
        &["COMMUNICATION", "SEND_MESSAGE", "DM"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<String>> {
        async move {
            let target = args["target"].as_str().ok_or_else(|| anyhow::anyhow!("Missing target block"))?;
            let message_body = args["message_body"].as_str().ok_or_else(|| anyhow::anyhow!("Missing message_body block"))?;

            info!("Communication: Resolving target JID for: {}", target);

            // 1. Dynamic ID Resolution
            let (resolved_jid, channel_type, conversation_id, user_uuid) = if let Ok(parsed_uuid) = uuid::Uuid::parse_str(target) {
                // 🅰️ TARGET IS A UUID: Resolve from database
                let row = sqlx::query!(
                    "SELECT c.channel_type, c.external_id, c.conversation_id, c.user_id 
                     FROM channels c 
                     WHERE c.user_id = $1 
                     ORDER BY c.created_at DESC LIMIT 1",
                    parsed_uuid
                ).fetch_optional(&dispatcher.pool).await?;

                match row {
                    Some(r) => (
                        r.external_id,
                        r.channel_type,
                        r.conversation_id.unwrap_or(uuid::Uuid::nil()),
                        r.user_id
                    ),
                    None => return Err(anyhow::anyhow!("No active channel JID mapped to UUID: {}", parsed_uuid)),
                }
            } else {
                // 🅱️/🅲 TARGET IS JID OR LID: Resolve channel info
                let row = sqlx::query!(
                    "SELECT channel_type, conversation_id, user_id FROM channels WHERE external_id = $1 OR external_chat_id= $1 LIMIT 1",
                    target
                ).fetch_optional(&dispatcher.pool).await?;

                match row {
                    Some(r) => (
                        target.to_string(),
                        r.channel_type,
                        r.conversation_id.unwrap_or(uuid::Uuid::nil()),
                        r.user_id
                    ),
                    None => {
                        // Minimal fallback if JID isn't in our channels table yet
                        (target.to_string(), "whatsapp".to_string(), uuid::Uuid::nil(), None)
                    }
                }
            };

            info!("Communication: Resolved to {} on {}", resolved_jid, channel_type);

            let sender_uuid = dispatcher.user_id.unwrap_or_default();
            let message_id = uuid::Uuid::new_v4();

            // 2. Log to Message Database (Context Retention)
            if conversation_id != uuid::Uuid::nil() {
                let pool = dispatcher.pool.clone();
                let msg_id = message_id.clone();
                let conv_id = conversation_id.clone();
                let content = message_body.to_string();

                tokio::spawn(async move {
                    let _ = sqlx::query!(
                        "INSERT INTO messages (id, conversation_id, user_id, role, content) VALUES ($1, $2, $3, $4, $5)",
                        msg_id, conv_id, sender_uuid, "assistant", content
                    ).execute(&pool).await;
                });
            }

            // 3. Dispatch to Channel Bridge via EMQX/MQTT
            let message_item = crate::feature::conversation::model::MessageItem {
                id: message_id,
                display_name: Some("Nomi".to_string()),
                conversation_id,
                role: "assistant".to_string(),
                content: message_body.to_string(),
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

            let members = user_uuid.map(|u| vec![u]).unwrap_or_default();

            crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber(
                &dispatcher.app_state,
                members,
                conversation_id,
                crate::feature::MessageSource::Multiple {
                    source: vec!["whatsapp".to_string(), "telegram".to_string(), "web".to_string()],
                },
                message_item.to_sse_json(0),
                message_item,
            ).await;

            let res_content = format!("Message successfully dispatched to target: {}", resolved_jid);
            let result = ToolResult {
                error: "".to_string(),
                success: true,
                content: res_content.clone(),
                follow_up_prompt: format!(
                    "The message has been sent to {}. Acknowledge this to the user and continue.",
                    target
                ),
            };

            Ok(serde_json::to_string(&result)?)
        }
            .boxed()
    }
}
