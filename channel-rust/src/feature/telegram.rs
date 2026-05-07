use teloxide::prelude::*;
use teloxide::types::{ChatAction, Recipient, MessageEntityKind};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use tracing::{error, info};
use crate::feature::InboundMessage;
use crate::common::redis::RedisClient;
use regex::Regex;

pub async fn start_telegram_worker(bot: Bot, redis: RedisClient) {
    info!("Starting Telegram worker dispatcher...");
    info!("Reminder: Please ensure 'Privacy Mode' is set to Enabled in BotFather for your bot. This ensures the bot only receives messages that start with / or @mention it, reducing unnecessary hits to our VPS.");

    let me = bot.get_me().await.expect("Failed to get bot info");
    let bot_username = me.user.username.clone().unwrap_or_default();

    let handler = Update::filter_message().endpoint(
        move |msg: Message, redis: RedisClient| {
            let bot_username_clone = bot_username.clone();
            async move {
                let original_text = msg.text().unwrap_or_default();
                let mut text = original_text.to_string();
                let chat_id = msg.chat.id.to_string();
                let user = msg.from();
                let sender_id = user.map(|u| u.id.to_string()).unwrap_or_else(|| chat_id.clone());
                let is_private = msg.chat.is_private();

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

                // Task 2: Handle Native Mentions
                let bot_mention_str = format!("@{}", bot_username_clone.to_lowercase());
                if original_text.to_lowercase().contains(&bot_mention_str) {
                    let mut native_mentioned = false;
                    if let Some(entities) = msg.entities() {
                        for entity in entities {
                            if matches!(entity.kind, MessageEntityKind::Mention | MessageEntityKind::TextMention { .. }) {
                                is_mentioned = true;
                                native_mentioned = true;
                            }
                        }
                    }
                    if native_mentioned && !is_private {
                        let bot_mention_regex = Regex::new(&format!(r"(?i){}\b", regex::escape(&bot_mention_str))).unwrap();
                        text = bot_mention_regex.replace_all(&text, "").to_string();
                    }
                }

                if !is_private && !is_mentioned {
                    return respond(());
                }

                text = text.trim().to_string();
                if text.is_empty() {
                    text = original_text.trim().to_string();
                }

                info!("Received Telegram message from {} in chat {}: {}", sender_id, chat_id, text);

                let display_name = user.map(|u| u.full_name()).unwrap_or_default();
                let username = user.and_then(|u| u.username.clone()).unwrap_or_default();
                let metadata = serde_json::json!({
                    "display_name": display_name,
                    "username": username
                });

                let inbound = InboundMessage {
                    sender_id,
                    chat_id,
                    text,
                    channel: "telegram".to_string(),
                    metadata: Some(metadata),
                };

                if let Err(e) = redis.publish_event("nomi:inbound", &inbound).await {
                    error!("Failed to publish Telegram inbound to Redis: {}", e);
                }

                respond(())
            }
        },
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![redis])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}


pub async fn send_telegram_message(bot: Bot, chat_id: String, text: String) -> anyhow::Result<()> {
    let chat_id = chat_id.parse::<i64>()?;
    bot.send_message(Recipient::Id(ChatId(chat_id)), text).await?;
    Ok(())
}

pub async fn send_telegram_typing(bot: Bot, chat_id: String) -> anyhow::Result<()> {
    let chat_id = chat_id.parse::<i64>()?;
    bot.send_chat_action(ChatId(chat_id), ChatAction::Typing).await?;
    Ok(())
}
