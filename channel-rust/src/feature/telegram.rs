use crate::common::redis::RedisClient;
use crate::common::storage::StorageClient;
use crate::feature::{InboundMessage, OutboundMessage};
use anyhow::anyhow;
use chrono::Utc;
use regex::Regex;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{ChatAction, FileId, InputFile, MessageEntityKind, Recipient};
use tokio::io::AsyncReadExt;
use tracing::{error, info};

pub async fn start_telegram_worker(bot: Bot, redis: RedisClient, storage_client: StorageClient) {
    info!("Starting Telegram worker dispatcher...");
    info!(
        "Reminder: Please ensure 'Privacy Mode' is set to Enabled in BotFather for your bot. This ensures the bot only receives messages that start with / or @mention it, reducing unnecessary hits to our VPS."
    );

    let me = bot.get_me().await.expect("Failed to get bot info");
    let bot_username = me.user.username.clone().unwrap_or_default();
    let bot_instance_clone = bot.clone();
    let handler = Update::filter_message().endpoint(move |msg: Message, redis: RedisClient| {
        let bot_username_clone = bot_username.clone();
        let storage = storage_client.clone();
        let bot_clone = bot_instance_clone.clone();
        async move {
            let original_text = msg.text().unwrap_or_default();
            let mut text = original_text.to_string();
            let conversation_id = msg.chat.id.to_string();
            let message_id = msg.id.to_string();
            let user = msg.from.clone();
            if let None = user {
                info!("sender is none");
                return respond(());
            }
            let user = user.unwrap();
            let sender_id = user.id.to_string();
            let is_private = msg.chat.is_private();
            let is_group = msg.chat.is_group();
            let mut image_url: Option<String> = None;
            let mut video_url: Option<String> = None;
            let mut audio_url: Option<String> = None;
            let mut doc_url: Option<String> = None;
            let mut sticker_url: Option<String> = None;

            if let Some(photos) = msg.photo() {
                info!("image detected");
                if let Some(highest_res) = photos.last() {
                    let file_id = &highest_res.file.id;
                    let extract = extract_and_upload_file_telegram(
                        &bot_clone,
                        &storage,
                        file_id.clone(),
                        format!("{}/{}", sender_id, message_id),
                    )
                    .await;

                    info!("result image upload {:?}", extract);
                    if let Ok(file) = extract {
                        info!("image uploaded");
                        image_url = Some(file);
                    }
                }
            }

            if let Some(video) = msg.video() {
                info!("video detected");
                let file_id = &video.file.id;

                let extract = extract_and_upload_file_telegram(
                    &bot_clone,
                    &storage,
                    file_id.clone(),
                    format!("{}/{}", sender_id, message_id),
                )
                .await;

                info!("result video upload {:?}", extract);
                if let Ok(file) = extract {
                    info!("video uploaded");
                    video_url = Some(file);
                }
            }

            if let Some(audio) = msg.audio() {
                info!("audio detected");
                let file_id = &audio.file.id;
                let extract = extract_and_upload_file_telegram(
                    &bot_clone,
                    &storage,
                    file_id.clone(),
                    format!("{}/{}", sender_id, message_id),
                )
                .await;
                info!("result audio upload {:?}", extract);
                if let Ok(file) = extract {
                    info!("audio uploaded");
                    audio_url = Some(file);
                }
            }

            if let Some(voice) = msg.voice() {
                info!("vn detected");
                let file_id = &voice.file.id;
                let extract = extract_and_upload_file_telegram(
                    &bot_clone,
                    &storage,
                    file_id.clone(),
                    format!("{}/{}", sender_id, message_id),
                )
                .await;
                info!("result vn upload {:?}", extract);
                if let Ok(file) = extract {
                    info!("vn uploaded");
                    audio_url = Some(file);
                }
            }

            if let Some(document) = msg.document() {
                info!("document detected");
                let file_id = &document.file.id;
                let extract = extract_and_upload_file_telegram(
                    &bot_clone,
                    &storage,
                    file_id.clone(),
                    format!("{}/{}", sender_id, message_id),
                )
                .await;
                info!("result document upload {:?}", extract);
                if let Ok(file) = extract {
                    info!("document uploaded");
                    doc_url = Some(file);
                }
            }

            if let Some(sticker) = msg.sticker() {
                info!("sticker detected");
                let file_id = &sticker.file.id;
                let extract = extract_and_upload_file_telegram(
                    &bot_clone,
                    &storage,
                    file_id.clone(),
                    format!("{}/{}", sender_id, message_id),
                )
                .await;
                info!("result sticker upload {:?}", extract);
                if let Ok(file) = extract {
                    info!("sticker uploaded");
                    sticker_url = Some(file);
                }
            }

            let mut is_mentioned = Regex::new(r"(?i)@?(nomi|nom\s*nom|nomnom|nomiii|nom)\b").unwrap().is_match(&text);


            // Task 2: Handle Native Mentions
            let bot_mention_str = format!("@{}", bot_username_clone.to_lowercase());
            if original_text.to_lowercase().contains(&bot_mention_str) {
                let mut native_mentioned = false;
                if let Some(entities) = msg.entities() {
                    for entity in entities {
                        if matches!(
                            entity.kind,
                            MessageEntityKind::Mention | MessageEntityKind::TextMention { .. }
                        ) {
                            is_mentioned = true;
                            native_mentioned = true;
                        }
                    }
                }
                if native_mentioned && !is_private {
                    let bot_mention_regex =
                        Regex::new(&format!(r"(?i){}\b", regex::escape(&bot_mention_str))).unwrap();
                    text = bot_mention_regex.replace_all(&text, "").to_string();
                }
            }

            text = text.trim().to_string();
            if text.is_empty() {
                text = original_text.trim().to_string();
            }

            info!(
                "Received Telegram message from {} in chat {}: {}",
                sender_id, conversation_id, text
            );

            let display_name = user.full_name();
            let username = user.username.clone().unwrap_or_default();
            let metadata = serde_json::json!({
                "display_name": display_name,
                "username": username
            });
            let original_meta = serde_json::json!(msg);

            let inbound = InboundMessage {
                message_id,
                is_group,
                is_private,
                is_mentioned,
                sender_id,
                conversation_id,
                text,
                channel: "telegram".to_string(),
                video_url,
                image_url,
                audio_url,
                doc_url,
                sticker_url,
                metadata: Some(metadata),
                original_meta:Some(original_meta)
            };

            info!("nomi:inbound => {:?}", inbound);
            if let Err(e) = redis.publish_event("nomi:inbound", &inbound).await {
                error!("Failed to publish Telegram inbound to Redis: {}", e);
            }

            respond(())
        }
    });

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![redis])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

pub async fn send_telegram_message(
    bot: Bot,
    msg: OutboundMessage,
    storage: &StorageClient,
    redis: &RedisClient,
) -> anyhow::Result<()> {
    if let Ok(chat_id) = msg.conversation_id.parse::<i64>() {
        if let Some(image) = msg.image_url.clone() {
            if let Err(err) = send_telegram_audio(storage, &bot, chat_id, image).await {
                let _ = redis
                    .publish_fallback(
                        format!("Failed to sent image reason:{}", err),
                        400,
                        Some(msg.clone()),
                    )
                    .await;
            }
        }

        if let Some(video) = msg.video_url.clone() {
            if let Err(err) = send_telegram_audio(storage, &bot, chat_id, video).await {
                let _ = redis
                    .publish_fallback(
                        format!("Failed to sent video reason:{}", err),
                        400,
                        Some(msg.clone()),
                    )
                    .await;
            }
        }

        if let Some(audio) = msg.audio_url.clone() {
            if let Err(err) = send_telegram_audio(storage, &bot, chat_id, audio).await {
                let _ = redis
                    .publish_fallback(
                        format!("Failed to sent audio reason:{}", err),
                        400,
                        Some(msg.clone()),
                    )
                    .await;
            }
        }

        if let Some(document) = msg.doc_url.clone() {
            if let Err(err) = send_telegram_document(storage, &bot, chat_id, document).await {
                let _ = redis
                    .publish_fallback(
                        format!("Failed to sent document reason:{}", err),
                        400,
                        Some(msg.clone()),
                    )
                    .await;
            }
        }

        if let Some(sticker) = msg.sticker_url.clone() {
            if let Err(err) = send_telegram_sticker(storage, &bot, chat_id, sticker).await {
                let _ = redis
                    .publish_fallback(
                        format!("Failed to sent sticker reason:{}", err),
                        400,
                        Some(msg.clone()),
                    )
                    .await;
            }
        }
        bot.send_message(Recipient::Id(ChatId(chat_id)), msg.text)
            .await?;
    }

    Ok(())
}

pub async fn send_telegram_typing(bot: Bot, chat_id: String) -> anyhow::Result<()> {
    let chat_id = chat_id.parse::<i64>()?;
    bot.send_chat_action(ChatId(chat_id), ChatAction::Typing)
        .await?;
    Ok(())
}

pub async fn extract_and_upload_file_telegram(
    bot: &Bot,
    storage: &StorageClient,
    file_id: FileId,
    target_file_path: String,
) -> Result<String, String> {
    let bucket = "conversations";
    let mut buff = Vec::new();
    match bot.get_file(file_id).await {
        Ok(bot_file) => {
            let ext = mime_guess::from_path(bot_file.path.clone()).first_or_octet_stream();
            let ext = mime_guess::get_mime_extensions(&ext)
                .unwrap()
                .first()
                .unwrap()
                .to_string();
            let file_name = format!("{}.{}", bot_file.id, ext);
            let temp_file = tokio::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(file_name.clone())
                .await;
            match temp_file {
                Ok(mut file_destination) => match bot
                    .download_file(&bot_file.path, &mut file_destination)
                    .await
                {
                    Ok(_) => match file_destination.read_to_end(&mut buff).await {
                        Ok(_) => match storage
                            .upload_byte(
                                bucket.to_string(),
                                format!("{}.{}", target_file_path, ext),
                                buff,
                            )
                            .await
                        {
                            Ok(upload_file) => {
                                info!("success uploaded");
                                let _ = tokio::fs::remove_file(file_name).await;
                                Ok(upload_file.clone())
                            }
                            Err(e) => Err(format!("Failed uploading file: {}", e)),
                        },

                        Err(e) => Err(format!("Failed read final temp file: {}", e)),
                    },
                    Err(e) => Err(format!("Failed download  file: {}", e)),
                },
                Err(e) => Err(format!("Failed download  file: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed download  file: {}", e)),
    }
}

pub async fn send_telegram_sticker(
    storage: &StorageClient,
    tele_client: &Bot,
    target: i64,
    sticker_path: String,
) -> anyhow::Result<String> {
    let get_sticker = storage
        .get_file("conversations".to_string(), sticker_path.clone())
        .await;

    if let Err(err) = get_sticker {
        info!("failed sent sticker message: {}", err);
        return Err(anyhow::anyhow!(err.to_string()));
    }

    let buff = get_sticker.unwrap().to_vec();
    let image = InputFile::memory(buff);
    let send_sticker = tele_client
        .send_sticker(Recipient::Id(ChatId(target)), image)
        .await;
    if let Err(err) = send_sticker {
        info!("sending sticker:{} failed: {}", sticker_path, err);
        return Err(anyhow!("Failed to send sticker"));
    }
    info!("sticker sent");
    Ok("sticker sent".to_string())
}

pub async fn send_telegram_document(
    storage: &StorageClient,
    tele_client: &Bot,
    target: i64,
    document_path: String,
) -> anyhow::Result<String> {
    let get_file = storage
        .get_file("conversations".to_string(), document_path.clone())
        .await;
    if let Err(err) = get_file {
        info!("failed sent document:{} failed: {}", document_path, err);
        return Err(anyhow::anyhow!(err.to_string()));
    }

    let doc = get_file.unwrap();

    let ext = mime_guess::from_path(document_path.clone()).first_or_octet_stream();
    let ext = mime_guess::get_mime_extensions(&ext)
        .unwrap()
        .first()
        .unwrap()
        .to_string();
    let buff = doc.to_vec();
    let image = InputFile::memory(buff).file_name(format!("{}.{}", Utc::now().to_rfc3339(), ext));
    let send = tele_client
        .send_document(Recipient::Id(ChatId(target)), image)
        .await;
    if let Err(err) = send {
        info!("failed sent document:{} failed: {}", document_path, err);
        return Err(anyhow::anyhow!(err.to_string()));
    }
    info!("document:{} sent", target);
    Ok("document:sent".to_string())
}

pub async fn send_telegram_audio(
    storage: &StorageClient,
    tele_client: &Bot,
    target: i64,
    audio_path: String,
) -> anyhow::Result<String> {
    let get_file = storage
        .get_file("conversations".to_string(), audio_path.clone())
        .await;
    if let Err(err) = get_file {
        info!("failed sent audio:{} failed: {}", audio_path, err);
        return Err(anyhow::anyhow!(err.to_string()));
    }
    let audio_url = get_file.unwrap();

    let ext = mime_guess::from_path(audio_path.clone()).first_or_octet_stream();
    let ext = mime_guess::get_mime_extensions(&ext)
        .unwrap()
        .first()
        .unwrap()
        .to_string();
    let buff = audio_url.to_vec();
    let image = InputFile::memory(buff).file_name(format!("{}.{}", Utc::now().to_rfc3339(), ext));
    let send = tele_client
        .send_audio(Recipient::Id(ChatId(target)), image)
        .await;
    if let Err(err) = send {
        info!("sending audio:{} failed: {}", audio_path, err);
        return Err(anyhow::anyhow!(err));
    }

    info!("audio:{} sent", target);
    Ok("audio:{} sent".to_string())
}

pub async fn send_telegram_video(
    storage: &StorageClient,
    tele_client: &Bot,
    target: i64,
    video_path: String,
) -> anyhow::Result<String> {
    let get_file = storage
        .get_file("conversations".to_string(), video_path.clone())
        .await;
    if let Err(err) = get_file {
        info!("Failed download  file: {}", err);
        return Err(anyhow::anyhow!(err.to_string()));
    }
    let video_url = get_file.unwrap();

    let ext = mime_guess::from_path(video_path.clone()).first_or_octet_stream();
    let ext = mime_guess::get_mime_extensions(&ext)
        .unwrap()
        .first()
        .unwrap()
        .to_string();
    let buff = video_url.to_vec();
    let image = InputFile::memory(buff).file_name(format!("{}.{}", Utc::now().to_rfc3339(), ext));
    let send = tele_client
        .send_video(Recipient::Id(ChatId(target)), image)
        .await;

    if let Err(err) = send {
        info!("sending video:{} failed: {}", video_path, err);
        return Err(anyhow::anyhow!(err));
    }

    info!("video:{} sent", target);
    Ok("video:{} sent".to_string())
}

pub async fn send_telegram_image(
    storage: &StorageClient,
    tele_client: &Bot,
    target: i64,
    image_path: String,
) -> anyhow::Result<String> {
    let get_file = storage
        .get_file("conversations".to_string(), image_path.clone())
        .await;
    if let Err(err) = get_file {
        info!("failed download  file: {}", err);
        return Err(anyhow::anyhow!(err.to_string()));
    }
    let image_url = get_file.unwrap();

    let ext = mime_guess::from_path(image_path.clone()).first_or_octet_stream();
    let ext = mime_guess::get_mime_extensions(&ext)
        .unwrap()
        .first()
        .unwrap()
        .to_string();
    let buff = image_url.to_vec();
    let image = InputFile::memory(buff).file_name(format!("{}.{}", Utc::now().to_rfc3339(), ext));
    let send = tele_client
        .send_photo(Recipient::Id(ChatId(target)), image)
        .await;
    if let Err(err) = send {
        info!("sending image:{} failed: {}", image_path, err);
        return Err(anyhow::anyhow!(err));
    }

    info!("image:{} sent", target);
    Ok("image:{} sent".to_string())
}
