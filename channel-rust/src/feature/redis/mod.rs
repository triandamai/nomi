use crate::feature::{InboundMessage, OutboundMessage, PresenceMessage};
use tracing::{error, info};
use tokio_stream::StreamExt;
use teloxide::prelude::*;
use crate::AppState;

pub async fn start_redis_listener(state: AppState) -> anyhow::Result<()> {
    let mut pubsub = state.redis.get_pubsub().await?;

    pubsub.subscribe("nomi:outbound").await?;
    pubsub.subscribe("nomi:presence").await?;

    info!("Redis listener started for nomi:outbound and nomi:presence");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let channel = msg.get_channel_name().to_string();
        let payload: String = msg.get_payload()?;
        info!("Received channel:{}  message: {}",channel, payload);

        match channel.as_str() {
            "nomi:outbound" => {
                let outbound: OutboundMessage = match serde_json::from_str(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to parse outbound message: {}", e);
                        continue;
                    }
                };
                let bot_clone = state.bot.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_outbound_message(bot_clone, outbound).await {
                        error!("Error handling outbound message: {}", e);
                    }
                });
            }
            "nomi:presence" => {
                let presence: PresenceMessage = match serde_json::from_str(&payload) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("Failed to parse presence message: {}", e);
                        continue;
                    }
                };
                let bot_clone = state.bot.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_presence_message(bot_clone, presence).await {
                        error!("Error handling presence message: {}", e);
                    }
                });
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_outbound_message(bot: Bot, msg: OutboundMessage) -> anyhow::Result<()> {
    match msg.channel.as_str() {
        "whatsapp" => {
            info!("Sending to WhatsApp: {:?}", msg.sender_id);
            // TODO: Implement WA sending
        },
        "telegram" => {
            info!("Sending to Telegram: {}", msg.chat_id);
            crate::feature::telegram::send_telegram_message(bot, msg.chat_id, msg.text).await?;
        },
        _ => error!("Unknown platform: {}", msg.channel),
    }
    Ok(())
}

async fn handle_presence_message(bot: Bot, msg: PresenceMessage) -> anyhow::Result<()> {
    if msg.status == "typing" {
        match msg.channel.as_str() {
            "telegram" => {
                crate::feature::telegram::send_telegram_typing(bot, msg.chat_id).await?;
            }
            _ => {}
        }
    }
    Ok(())
}
