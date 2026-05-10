use crate::AppState;
use crate::common::redis::RedisClient;
use crate::common::storage::StorageClient;
use crate::feature::{InboundMessage, OutboundMessage};
use regex::Regex;
use image::GenericImageView;
use std::str::FromStr;
use std::sync::Arc;
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
    qr_code: Arc<Mutex<Option<String>>>,
    redis: RedisClient,
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

                                info!("nomi:inbound => {:?}",&inbound);
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
                qr_code,
                redis,
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
            info!("Processing sticker for WhatsApp: {}", path);
            let mut success = false;
            if let Ok(bytes) = storage.get_file("conversations".to_string(), path.clone()).await {
                // Load the image
                if let Ok(img) = image::load_from_memory(&bytes.to_vec()) {
                    // Create a 512x512 transparent background
                    let mut final_img = image::ImageBuffer::new(512, 512);

                    // Resize original image to fit in 512x512 while maintaining aspect ratio
                    let resized = img.resize(512, 512, image::imageops::FilterType::Lanczos3);
                    let (rw, rh) = resized.dimensions();

                    // Center it
                    let x = (512 - rw) / 2;
                    let y = (512 - rh) / 2;
                    
                    // Convert resized to RGBA8 to ensure compatibility with final_img
                    let resized_rgba = resized.to_rgba8();
                    image::imageops::overlay(&mut final_img, &resized_rgba, x.into(), y.into());

                    // Convert to DynamicImage for WebP encoder
                    let dynamic_final = image::DynamicImage::ImageRgba8(final_img);

                    // Convert to WebP
                    if let Ok(encoder) = webp::Encoder::from_image(&dynamic_final) {
                        let webp_data = encoder.encode(80.0).to_vec();

                        // Upload to WhatsApp servers
                        if let Ok(upload) = self.client.upload(webp_data.clone(), MediaType::Sticker).await {
                             let mut payload = Message::default();
                             payload.sticker_message = Some(Box::new(StickerMessage {
                                url: Some(upload.url),
                                mimetype: Some("image/webp".to_string()),
                                file_sha256: Some(upload.file_sha256),
                                file_enc_sha256: Some(upload.file_enc_sha256),
                                media_key: Some(upload.media_key),
                                file_length: Some(webp_data.len() as u64),
                                ..Default::default()
                            }));
                            let _ = self.client.send_message(chat.clone(), payload).await;
                            let _ = tokio::time::sleep(Duration::from_secs(2)).await;
                            success = true;
                        } else {
                            error!("Failed to upload sticker to WhatsApp");
                        }
                    } else {
                        error!("Failed to encode image to WebP");
                    }
                } else {
                    error!("Failed to load image from memory for sticker");
                }
            } else {
                error!("Failed to get file from storage for sticker: {}", path);
            }

            if !success {
                let _ = self.redis.publish_event("nomi:outbound", &OutboundMessage {
                    is_group: msg.is_group,
                    sender_id: msg.sender_id.clone(),
                    conversation_id: msg.conversation_id.clone(),
                    text: "Sorry, Nomi couldn't turn that specific image into a sticker! 🏍️💨".to_string(),
                    channel: "whatsapp".to_string(),
                    user_id: None,
                    video_url: None,
                    image_url: None,
                    audio_url: None,
                    doc_url: None,
                    sticker_url: None,
                    metadata: None,
                }).await;
            }
        }

        let mut payload = Message::default();
        payload.conversation = Some(formatted_text);
        self.client.send_message(chat.clone(), payload).await?;
        Ok(())
    }

    pub async fn regenerate(&self, state: &AppState) -> anyhow::Result<()> {
        info!("Regenarate qr wa...");

        Ok(())
    }
    pub async fn logout(&self, state: &AppState) -> anyhow::Result<()> {
        info!("Logging out from WhatsApp...");
        // let _ = state.wa_cmd_tx.send(WhatsAppCommand::Logout);
        let _ = self.client.disconnect().await;
        let _ = tokio::fs::remove_file("/data/whatsapp.db").await;
        let _ = tokio::fs::remove_file("/data/whatsapp.db-shmb").await;
        let _ = tokio::fs::remove_file("/data/whatsapp.db-wal").await;
        let mut qr_lock = self.qr_code.lock().await;
        *qr_lock = None;
        Ok(())
    }
}
