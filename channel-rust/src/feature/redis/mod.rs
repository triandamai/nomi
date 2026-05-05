use crate::feature::{InboundMessage, OutboundMessage};
use redis::AsyncCommands;
use tracing::{error, info};
use tokio_stream::StreamExt;

pub async fn start_redis_listener() -> anyhow::Result<()> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    let mut pubsub = client.get_async_pubsub().await?;

    pubsub.subscribe("nomi:outbound").await?;

    info!("Redis listener started for nomi:outbound");

    let mut stream = pubsub.on_message();

    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        let outbound: OutboundMessage = match serde_json::from_str(&payload) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to parse outbound message: {}", e);
                continue;
            }
        };

        tokio::spawn(async move {
            if let Err(e) = handle_outbound_message(outbound).await {
                error!("Error handling outbound message: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_outbound_message(msg: OutboundMessage) -> anyhow::Result<()> {
    info!("Handling outbound to {}: {}", msg.external_id, msg.content);
    
    // TODO: Route to appropriate platform (WhatsApp or Telegram)
    // This will require access to the WA/Tele clients
    
    match msg.platform.as_str() {
        "whatsapp" => {
            info!("Sending to WhatsApp: {}", msg.external_id);
            // client.send_message(msg.external_id, msg.content).await?;
        },
        "telegram" => {
            info!("Sending to Telegram: {}", msg.external_id);
            // bot.send_message(msg.external_id, msg.content).await?;
        },
        _ => error!("Unknown platform: {}", msg.platform),
    }

    Ok(())
}

pub async fn publish_inbound(msg: InboundMessage) -> anyhow::Result<()> {
    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    
    let payload = serde_json::to_string(&msg)?;
    conn.publish::<&str, String, ()>("nomi:inbound", payload).await?;
    
    Ok(())
}
