use crate::feature::{FallBackPayload, OutboundMessage};
use redis::AsyncCommands;
use serde::Serialize;
use tracing::error;
use tracing::log::info;

#[derive(Clone)]
pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        Ok(Self { client })
    }

    pub async fn get_connection(&self) -> anyhow::Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                anyhow::anyhow!(e)
            })
    }

    pub async fn publish_event<T: Serialize>(
        &self,
        channel: &str,
        payload: &T,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let json = serde_json::to_string(payload)?;
        let _: () = conn.publish(channel, json).await.map_err(|e| {
            error!("Failed to publish to {}: {}", channel, e);
            anyhow::anyhow!(e)
        })?;
        Ok(())
    }

    pub async fn publish_fallback(
        &self,
        message: String,
        error: i32,
        payload: Option<OutboundMessage>,
    ) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        info!("sending nomi:channel-fallback message: {}", message);
        let json = FallBackPayload {
            payload,
            error: Some(message),
            code: error,
        };
        let _: () = conn
            .publish("nomi:channel-fallback", serde_json::to_string(&json)?)
            .await
            .map_err(|e| {
                error!("Failed to publish to {}: nomi:channel-fallback",  e);
                anyhow::anyhow!(e)
            })?;
        Ok(())
    }

    pub async fn get_pubsub(&self) -> anyhow::Result<redis::aio::PubSub> {
        self.client.get_async_pubsub().await.map_err(|e| {
            error!("Failed to get Redis pubsub: {}", e);
            anyhow::anyhow!(e)
        })
    }
}
