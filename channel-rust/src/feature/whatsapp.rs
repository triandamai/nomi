use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use wa_rs::bot::Bot;
use wa_rs_sqlite_storage::SqliteStore;
use wa_rs_tokio_transport::TokioWebSocketTransportFactory;
use wa_rs_ureq_http::UreqHttpClient;
use wa_rs::types::events::Event;
use wa_rs::Client;
use wa_rs::Jid;
use wa_rs::wa_rs_proto::whatsapp::Message;
use std::str::FromStr;
use regex::Regex;

use crate::common::redis::RedisClient;
use crate::feature::InboundMessage;

pub struct WhatsAppWorker {
    client: Arc<Client>,
    redis: RedisClient,
    qr_code: Arc<Mutex<Option<String>>>,
}

impl WhatsAppWorker {
    pub async fn new(
        db_path: &str,
        redis: RedisClient,
        qr_code: Arc<Mutex<Option<String>>>,
    ) -> anyhow::Result<(Self, Bot)> {
        let storage = Arc::new(SqliteStore::new(db_path).await?);
        
        let redis_clone = redis.clone();
        let qr_clone = qr_code.clone();

        let bot = Bot::builder()
            .with_backend(storage)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, client| {
                let redis = redis_clone.clone();
                let qr = qr_clone.clone();
                async move {
                    match event {
                        Event::PairingQrCode { code, .. } => {
                            info!("New WhatsApp QR Code received");
                            let mut qr_lock = qr.lock().await;
                            *qr_lock = Some(code);
                        }
                        Event::Connected(_) => {
                            info!("WhatsApp connected successfully!");
                            let mut qr_lock = qr.lock().await;
                            *qr_lock = None;
                        }
                        Event::Message(msg, info) => {
                            if info.source.is_from_me {
                                return;
                            }

                            let original_text = msg.conversation.clone().or(msg.extended_text_message.as_ref().and_then(|m| m.text.clone()));
                            
                            if let Some(original_text) = original_text {
                                let mut text = original_text.clone();
                                let sender_id = info.source.sender.to_string();
                                let chat_id = info.source.chat.to_string();
                                let is_group = chat_id.ends_with("@g.us");
                                let is_private = !is_group;

                                let mut is_mentioned = false;

                                // Task 1: The Mention Gate
                                let keyword_regex = Regex::new(r"(?i)@?(nomi|nom\s*nom|nomnom|nomiii|nom)\b").unwrap();
                                if keyword_regex.is_match(&text) {
                                    is_mentioned = true;
                                    // Task 3: Clean the Input
                                    if !is_private {
                                        text = keyword_regex.replace_all(&text, "").to_string();
                                    }
                                }

                                // Task 2: Handle Native Mentions (WhatsApp)
                                let mentioned_jids = msg.extended_text_message.as_ref()
                                    .and_then(|m| m.context_info.as_ref())
                                    .map(|c| c.mentioned_jid.clone())
                                    .unwrap_or_default();
                                
                                if let Some(own_jid) = client.get_pn().await {
                                    let own_jid_str = own_jid.to_string();
                                    if mentioned_jids.contains(&own_jid_str) {
                                        is_mentioned = true;
                                        if !is_private {
                                            let jid_user = own_jid_str.split('@').next().unwrap_or("");
                                            let mention_regex = Regex::new(&format!(r"(?i)@{}\b", regex::escape(jid_user))).unwrap();
                                            text = mention_regex.replace_all(&text, "").to_string();
                                        }
                                    }
                                }

                                if !is_private && !is_mentioned {
                                    return;
                                }

                                text = text.trim().to_string();
                                if text.is_empty() {
                                    text = original_text.trim().to_string();
                                }

                                info!("Received WhatsApp message from {}: {} \n", sender_id, text);

                                let display_name = info.push_name.clone();
                                let phone_number = info.source.sender.to_string().split('@').next().unwrap_or("").to_string();
                                let metadata = serde_json::json!({
                                    "display_name": display_name,
                                    "phone_number": phone_number
                                });

                                let inbound = InboundMessage {
                                    is_group:!is_private,
                                    sender_id,
                                    chat_id,
                                    text,
                                    channel: "whatsapp".to_string(),
                                    metadata: Some(metadata),
                                };

                                if let Err(e) = redis.publish_event("nomi:inbound", &inbound).await {
                                    error!("Failed to publish WhatsApp inbound to Redis: {}", e);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            })
            .build()
            .await?;

        let client = bot.client().clone();
        
        Ok((Self {
            client,
            redis,
            qr_code,
        }, bot))
    }

    pub async fn send_message(&self, chat_id: String, text: String) -> anyhow::Result<()> {
        let chat = Jid::from_str(&chat_id).map_err(|e| anyhow::anyhow!("Invalid chat id: {}", e))?;
        let formatted_text = crate::common::format::markdown_to_whatsapp(&text);
        
        let mut msg = Message::default();
        msg.conversation = Some(formatted_text);

        self.client.send_message(chat, msg).await?;
        Ok(())
    }

    pub async fn logout(&self) -> anyhow::Result<()> {
        info!("Logging out from WhatsApp...");
        self.client.disconnect().await;
        // self.client.logout().await.map_err(|e| anyhow::anyhow!("Logout failed: {}", e))?;
        let mut qr_lock = self.qr_code.lock().await;
        *qr_lock = None;
        Ok(())
    }
}
