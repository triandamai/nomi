use teloxide::prelude::*;
use teloxide::types::{ChatAction, Recipient};
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use tracing::{error, info};
use crate::feature::InboundMessage;
use crate::common::redis::RedisClient;

pub async fn start_telegram_worker(bot: Bot, redis: RedisClient) {
    info!("Starting Telegram worker dispatcher...");

    let handler = Update::filter_message().endpoint(
        |msg: Message, redis: RedisClient| async move {
            let text = msg.text().unwrap_or_default();
            let chat_id = msg.chat.id.to_string();
            let user = msg.from();
            let sender_id = user.map(|u| u.id.to_string()).unwrap_or_else(|| chat_id.clone());

            info!("Received Telegram message from {} in chat {}: {}", sender_id, chat_id, text);

            let inbound = InboundMessage {
                sender_id,
                chat_id,
                text: text.to_string(),
                channel: "telegram".to_string(),
            };

            if let Err(e) = redis.publish_event("nomi:inbound", &inbound).await {
                error!("Failed to publish Telegram inbound to Redis: {}", e);
            }

            respond(())
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
