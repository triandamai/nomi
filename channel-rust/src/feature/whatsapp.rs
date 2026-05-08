use crate::AppState;
use crate::common::redis::RedisClient;
use crate::common::storage::StorageClient;
use crate::feature::{InboundMessage, OutboundMessage, WhatsAppCommand};
use dotenvy::var;
use regex::Regex;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};
use wa_rs::Client;
use wa_rs::Jid;
use wa_rs::bot::Bot;
use wa_rs::types::events::Event;
use wa_rs::wa_rs_proto::whatsapp::Message;
use wa_rs::wa_rs_proto::whatsapp::message::{
    AudioMessage, DocumentMessage, ImageMessage, StickerMessage, VideoMessage,
};
use wa_rs_core::download::MediaType;
use wa_rs_sqlite_storage::SqliteStore;
use wa_rs_tokio_transport::TokioWebSocketTransportFactory;
use wa_rs_ureq_http::UreqHttpClient;

pub struct WhatsAppWorker {
    client: Arc<Client>,
    redis: RedisClient,
    storage: StorageClient,
    qr_code: Arc<Mutex<Option<String>>>,
}

impl WhatsAppWorker {
    pub async fn new(
        db_path: &str,
        redis: RedisClient,
        storage_client: StorageClient,
        qr_code: Arc<Mutex<Option<String>>>,
    ) -> anyhow::Result<(Self, Bot)> {
        let local_db = Arc::new(SqliteStore::new(db_path).await?);

        let redis_clone = redis.clone();
        let qr_clone = qr_code.clone();
        let storage_clone = storage_client.clone();

        let bot = Bot::builder()
            .with_backend(local_db)
            .with_transport_factory(TokioWebSocketTransportFactory::new())
            .with_http_client(UreqHttpClient::new())
            .on_event(move |event, client| {
                let redis = redis_clone.clone();
                let qr = qr_clone.clone();
                let storage = storage_clone.clone();
                async move {
                    match event {
                        Event::LoggedOut {
                            ..
                        } => {
                            info!("logged out from whatsapp");
                        }
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

                            let sender_id = info.source.sender.to_string();
                            let conversation_id = info.source.chat.to_string();
                            let message_id = info.id.to_string();
                            let is_group = conversation_id.ends_with("@g.us");
                            let is_private = !is_group;
                            let mut image_url: Option<String> = None;
                            let mut video_url: Option<String> = None;
                            let mut audio_url: Option<String> = None;
                            let mut doc_url: Option<String> = None;
                            let mut sticker_url: Option<String> = None;
                            let bucket = "conversations";

                            let mut is_mentioned = false;
                            if let Some(img) = &msg.image_message {
                                info!("image detected");
                                if let Ok(data) = client.download(img.as_ref()).await {
                                    let ext = mime_guess::get_mime_extensions_str(img.mimetype())
                                        .and_then(|exts| exts.first())
                                        .unwrap_or(&".jpg");
                                    if let Ok(upload_image) = &storage
                                        .upload_byte(
                                            bucket.to_string(),
                                            format!("{}/{}.{}", sender_id, conversation_id, ext),
                                            data,
                                        )
                                        .await
                                    {
                                        info!("image uploaded");
                                        image_url = Some(upload_image.clone());
                                    }
                                }
                            }
                            // Video
                            if let Some(video) = &msg.video_message {
                                info!("video detected");
                                if let Ok(data) = client.download(video.as_ref()).await {
                                    let ext = mime_guess::get_mime_extensions_str(video.mimetype())
                                        .and_then(|exts| exts.first())
                                        .unwrap_or(&".mp4");
                                    if let Ok(upload_video) = &storage
                                        .upload_byte(
                                            bucket.to_string(),
                                            format!("{}/{}.{}", sender_id, conversation_id, ext),
                                            data,
                                        )
                                        .await
                                    {
                                        info!("video uploaded");
                                        video_url = Some(upload_video.clone());
                                    }
                                }
                            }

                            // Audio
                            if let Some(audio) = &msg.audio_message {
                                info!("audio detected");
                                if let Ok(data) = client.download(audio.as_ref()).await {
                                    let ext = if audio.ptt() { "ogg" } else { "mp3" };
                                    if let Ok(upload_audio) = &storage
                                        .upload_byte(
                                            bucket.to_string(),
                                            format!("{}/{}.{}", sender_id, conversation_id, ext),
                                            data,
                                        )
                                        .await
                                    {
                                        info!("audio uploaded");
                                        audio_url = Some(upload_audio.clone());
                                    }
                                }
                            }

                            // Document
                            if let Some(doc) = &msg.document_message {
                                info!("document detected");
                                if let Ok(data) = client.download(doc.as_ref()).await {
                                    let filename = doc.file_name.as_deref().unwrap_or("document");
                                    if let Ok(upload_doc) = &storage
                                        .upload_byte(
                                            bucket.to_string(),
                                            format!(
                                                "{}/{}/{}",
                                                sender_id, conversation_id, filename
                                            ),
                                            data,
                                        )
                                        .await
                                    {
                                        info!("document uploaded");
                                        doc_url = Some(upload_doc.clone());
                                    }
                                };
                            }

                            // Sticker
                            if let Some(sticker) = &msg.sticker_message {
                                info!("sticker detected");
                                if let Ok(data) = client.download(sticker.as_ref()).await {
                                    let ext =
                                        mime_guess::get_mime_extensions_str(sticker.mimetype())
                                            .and_then(|exts| exts.first())
                                            .unwrap_or(&".mp4");
                                    if let Ok(upload_sticker) = &storage
                                        .upload_byte(
                                            bucket.to_string(),
                                            format!("{}/{}.{}", sender_id, conversation_id, ext),
                                            data,
                                        )
                                        .await
                                    {
                                        info!("sticker uploaded");
                                        sticker_url = Some(upload_sticker.clone());
                                    }
                                }
                            }

                            let original_text = msg.conversation.clone().or(msg
                                .extended_text_message
                                .as_ref()
                                .and_then(|m| m.text.clone()));

                            if let Some(original_text) = original_text {
                                let mut text = original_text.clone();

                                // Task 1: The Mention Gate
                                let keyword_regex =
                                    Regex::new(r"(?i)@?(nomi|nom\s*nom|nomnom|nomiii|nom)\b")
                                        .unwrap();
                                if keyword_regex.is_match(&text) {
                                    is_mentioned = true;
                                    // Task 3: Clean the Input
                                    if !is_private {
                                        text = keyword_regex.replace_all(&text, "").to_string();
                                    }
                                }

                                // Task 2: Handle Native Mentions (WhatsApp)
                                let mentioned_jids = msg
                                    .extended_text_message
                                    .as_ref()
                                    .and_then(|m| m.context_info.as_ref())
                                    .map(|c| c.mentioned_jid.clone())
                                    .unwrap_or_default();

                                if let Some(own_jid) = client.get_pn().await {
                                    let own_jid_str = own_jid.to_string();
                                    if mentioned_jids.contains(&own_jid_str) {
                                        is_mentioned = true;
                                        if !is_private {
                                            let jid_user =
                                                own_jid_str.split('@').next().unwrap_or("");
                                            let mention_regex = Regex::new(&format!(
                                                r"(?i)@{}\b",
                                                regex::escape(jid_user)
                                            ))
                                            .unwrap();
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
                                let phone_number = info
                                    .source
                                    .sender
                                    .to_string()
                                    .split('@')
                                    .next()
                                    .unwrap_or("")
                                    .to_string();

                                let metadata = serde_json::json!({
                                    "display_name": display_name,
                                    "phone_number": phone_number
                                });

                                let inbound = InboundMessage {
                                    is_group: !is_private,
                                    is_private,
                                    sender_id,
                                    conversation_id,
                                    message_id,
                                    text,
                                    video_url,
                                    image_url,
                                    doc_url,
                                    audio_url,
                                    sticker_url,
                                    channel: "whatsapp".to_string(),
                                    metadata: Some(metadata),
                                };

                                if let Err(e) = redis.publish_event("nomi:inbound", &inbound).await
                                {
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

        Ok((
            Self {
                client,
                redis,
                storage: storage_client,
                qr_code,
            },
            bot,
        ))
    }

    pub async fn send_message(
        &self,
        msg: OutboundMessage,
        storage: &StorageClient,
    ) -> anyhow::Result<()> {
        let chat = Jid::from_str(&msg.conversation_id)
            .map_err(|e| anyhow::anyhow!("Invalid chat id: {}", e))?;
        let formatted_text = crate::common::format::markdown_to_whatsapp(&msg.text);

        if let Some(path) = msg.image_url {
            let mut payload = Message::default();
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path).await {
                if let Ok(upload) = self.client.upload(bytes.to_vec(), MediaType::Image).await {
                    payload.image_message = Some(Box::new(ImageMessage {
                        url: Some(upload.url),
                        mimetype: Some(mime.to_string()),
                        ..Default::default()
                    }));
                    let _ = self.client.send_message(chat.clone(), payload).await;
                    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        if let Some(path) = msg.video_url {
            let mut payload = Message::default();
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path).await {
                if let Ok(upload) = self.client.upload(bytes.to_vec(), MediaType::Video).await {
                    payload.video_message = Some(Box::new(VideoMessage {
                        url: Some(upload.url),
                        mimetype: Some(mime.to_string()),
                        ..Default::default()
                    }));
                    let _ = self.client.send_message(chat.clone(), payload).await;
                    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        if let Some(path) = msg.audio_url {
            let mut payload = Message::default();
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path).await {
                if let Ok(upload) = self.client.upload(bytes.to_vec(), MediaType::Audio).await {
                    payload.audio_message = Some(Box::new(AudioMessage {
                        url: Some(upload.url),
                        mimetype: Some(mime.to_string()),
                        ..Default::default()
                    }));
                    let _ = self.client.send_message(chat.clone(), payload).await;
                    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        if let Some(path) = msg.doc_url {
            let mut payload = Message::default();
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path).await {
                if let Ok(upload) = self
                    .client
                    .upload(bytes.to_vec(), MediaType::Document)
                    .await
                {
                    payload.document_message = Some(Box::new(DocumentMessage {
                        url: Some(upload.url),
                        mimetype: Some(mime.to_string()),
                        ..Default::default()
                    }));
                    let _ = self.client.send_message(chat.clone(), payload).await;
                    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        if let Some(path) = msg.sticker_url {
            let mut payload = Message::default();
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path).await {
                if let Ok(upload) = self.client.upload(bytes.to_vec(), MediaType::Sticker).await {
                    payload.sticker_message = Some(Box::new(StickerMessage {
                        url: Some(upload.url),
                        mimetype: Some(mime.to_string()),
                        ..Default::default()
                    }));
                    let _ = self.client.send_message(chat.clone(), payload).await;
                    let _ = tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }

        let mut payload = Message::default();
        payload.conversation = Some(formatted_text);
        self.client.send_message(chat.clone(), payload).await?;
        Ok(())
    }

    pub async fn logout(&self, state: &AppState) -> anyhow::Result<()> {
        info!("Logging out from WhatsApp...");
        let _ = state.wa_cmd_tx.send(WhatsAppCommand::Logout);
        // self.client.logout().await.map_err(|e| anyhow::anyhow!("Logout failed: {}", e))?;
        let mut qr_lock = self.qr_code.lock().await;
        *qr_lock = None;
        Ok(())
    }
}
