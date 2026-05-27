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
            "description": "Fires an outbound message payload to a specified target user by their UUID. Resolves registered communication channels (WhatsApp, Telegram) dynamically.",
            "parameters": {
                "type": "object",
                "properties": {
                    "user_id": {
                        "type": "string",
                        "description": "The unique database UUID string of the target user."
                    },
                    "message_body": {
                        "type": "string",
                        "description": "The exact textual content of the outbound message to transmit."
                    },
                    "channel_type": {
                        "type": "string",
                        "enum": ["whatsapp", "telegram", "web", "mobile", "both"],
                        "description": "Optional. The communication channel to use. Required if the user has multiple registered database channels."
                    }
                },
                "required": ["user_id", "message_body"]
            }
        })
    }

    fn rules(&self) -> &str {
        "\n##OUTBOUND MESSAGING CORE RULES:\n\
        1. When instructed to send a message to a user, you MUST always call the user search tool (`manage_user` with action 'search') first to obtain the target user's database UUID.\n\
        2. Pass that database UUID directly into the 'user_id' field of this tool.\n\
        3. If the user has multiple registered channels (e.g. WhatsApp and Telegram) and you did not specify a 'channel_type', the tool will ask for clarification. In that case, choose the correct channel or ask the human user."
    }

    fn matching_intents(&self) -> &[&str] {
        &["COMMUNICATION", "SEND_MESSAGE", "DM"]
    }

    fn execute<'a>(
        &'a self,
        dispatcher: &'a ToolDispatcher,
        args: Value,
    ) -> BoxFuture<'a, anyhow::Result<ToolResult>> {
        async move {
            let user_id_str = args["user_id"].as_str().ok_or_else(|| anyhow::anyhow!("Missing user_id parameter"))?;
            let message_body = args["message_body"].as_str().ok_or_else(|| anyhow::anyhow!("Missing message_body parameter"))?;
            let req_channel_type = args["channel_type"].as_str();

            info!("Communication: Resolving channels for target user UUID: {}", user_id_str);

            let parsed_uuid = match uuid::Uuid::parse_str(user_id_str.trim()) {
                Ok(uid) => uid,
                Err(_) => {
                    return Ok(ToolResult {
                        error: format!("Invalid user_id format: '{}'. It must be a valid 36-character UUID string.", user_id_str),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: "The target user_id parameter must be a valid UUID. Please perform a user search first to retrieve the correct UUID.".to_string(),
                        ref_id: "".to_string(),
                    });
                }
            };

            // Query all channels registered to this user_id
            #[derive(sqlx::FromRow, Clone)]
            struct ChannelRow {
                channel_type: String,
                external_id: String,
                conversation_id: Option<uuid::Uuid>,
            }

            let db_channels = sqlx::query_as::<_, ChannelRow>(
                "SELECT channel_type, external_id, conversation_id \
                 FROM channels WHERE user_id = $1"
            )
            .bind(parsed_uuid)
            .fetch_all(&dispatcher.pool)
            .await?;

            let mut channels = db_channels.clone();

            if channels.is_empty() {
                info!("Communication: Target user has no WhatsApp/Telegram channels. Falling back to web/mobile virtual channels.");
                
                // Resolve active private/sub-chat conversation between Nomi and the target user
                let conversation_id = match sqlx::query_scalar::<_, uuid::Uuid>(
                    "SELECT c.id FROM conversations c \
                     INNER JOIN conversation_members cm ON c.id = cm.conversation_id \
                     WHERE cm.user_id = $1 AND (c.conversation_type = 'private' OR c.conversation_type = 'channel_subchat') \
                     LIMIT 1"
                )
                .bind(parsed_uuid)
                .fetch_optional(&dispatcher.pool)
                .await? {
                    Some(cid) => cid,
                    None => {
                        // Spawn a new private conversation if none exists
                        let new_cid = uuid::Uuid::new_v4();
                        let _ = sqlx::query(
                            "INSERT INTO conversations (id, title, conversation_type) VALUES ($1, 'Private Chat', 'private')"
                        )
                        .bind(new_cid)
                        .execute(&dispatcher.pool)
                        .await;
                        
                        let _ = sqlx::query(
                            "INSERT INTO conversation_members (conversation_id, user_id) VALUES ($1, $2)"
                        )
                        .bind(new_cid)
                        .bind(parsed_uuid)
                        .execute(&dispatcher.pool)
                        .await;
                        
                        new_cid
                    }
                };

                channels = vec![
                    ChannelRow {
                        channel_type: "web".to_string(),
                        external_id: "web".to_string(),
                        conversation_id: Some(conversation_id),
                    },
                    ChannelRow {
                        channel_type: "mobile".to_string(),
                        external_id: "mobile".to_string(),
                        conversation_id: Some(conversation_id),
                    },
                ];
            }

            // Identify candidate channels
            let selected_channels: Vec<ChannelRow> = if let Some(ch_type) = req_channel_type {
                let lower_type = ch_type.to_lowercase();
                let filtered: Vec<ChannelRow> = if lower_type == "both" {
                    channels.clone()
                } else {
                    channels.iter().filter(|c| c.channel_type.to_lowercase() == lower_type).cloned().collect()
                };

                if filtered.is_empty() {
                    let available: Vec<String> = channels.iter().map(|c| c.channel_type.clone()).collect();
                    return Ok(ToolResult {
                        error: format!("Requested channel_type '{}' is not registered for this user.", ch_type),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: format!("The requested channel is unavailable. The user only has the following channel(s) registered: {:?}. Please select one of these or specify 'both'.", available),
                        ref_id: "".to_string(),
                    });
                }
                filtered
            } else {
                // If there's more than one active database communication channel (WhatsApp/Telegram), ask for clarification!
                if db_channels.len() > 1 {
                    let available_types: Vec<String> = db_channels.iter().map(|c| c.channel_type.clone()).collect();
                    return Ok(ToolResult {
                        error: "Multiple communication channels available".to_string(),
                        success: false,
                        content: "".to_string(),
                        follow_up_prompt: format!(
                            "The target user has multiple registered communication channels: {:?}. \
                             Please clarify which channel you want to send the message to, or specify 'both'. \
                             Use the optional 'channel_type' parameter in your next send_message tool call.",
                            available_types
                        ),
                        ref_id: "".to_string(),
                    });
                }
                channels.clone()
            };

            let sender_uuid = dispatcher.user_id.unwrap_or_default();
            let mut sent_jids = Vec::new();
            let mut last_message_id = uuid::Uuid::new_v4();

            for channel in selected_channels {
                let resolved_jid = channel.external_id.clone();
                let channel_type = channel.channel_type.clone();
                let conversation_id = channel.conversation_id.unwrap_or(uuid::Uuid::nil());
                let message_id = uuid::Uuid::new_v4();
                last_message_id = message_id;

                info!("Communication: Dispatched message to {} on channel {}", resolved_jid, channel_type);
                sent_jids.push(resolved_jid.clone());

                // 2. Log to Message Database (Context Retention)
                if conversation_id != uuid::Uuid::nil() {
                    let pool = dispatcher.pool.clone();
                    let msg_id = message_id.clone();
                    let conv_id = conversation_id.clone();
                    let content = message_body.to_string();

                    tokio::spawn(async move {
                        let _ = sqlx::query(
                            "INSERT INTO messages (id, conversation_id, user_id, role, content, reply_to_id) VALUES ($1, $2, $3, $4, $5, NULL)"
                        )
                        .bind(msg_id)
                        .bind(conv_id)
                        .bind(sender_uuid)
                        .bind("assistant")
                        .bind(content)
                        .execute(&pool).await;
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
                    metadata: None,
                    reply_to_id: None,
                    replied_message: None,
                };

                let members = vec![parsed_uuid];

                crate::feature::message_processor::v2_orchestrator::send_message_to_subscriber(
                    &dispatcher.app_state,
                    members,
                    conversation_id,
                    crate::feature::MessageSource::Multiple {
                        source: vec![channel_type.clone(), "web".to_string()],
                    },
                    message_item.to_sse_json(0),
                    message_item,
                ).await;
            }

            let first_resolved_jid = sent_jids.first().cloned().unwrap_or_default();
            let res_content = format!("Message successfully dispatched to target channels: {:?}", sent_jids);
            
            // Encode target resolved JID inside ref_id for background loop (Phase 2 isolation)
            let compound_ref_id = format!("JID:{}|MSG:{}", first_resolved_jid, last_message_id);

            Ok(ToolResult {
                error: "".to_string(),
                success: true,
                content: res_content.clone(),
                follow_up_prompt: format!(
                    "The message has been sent to user UUID {} over target channels: {:?}. Acknowledge this to the user.",
                    user_id_str,
                    sent_jids
                ),
                ref_id: compound_ref_id,
            })
        }
        .boxed()
    }
}
