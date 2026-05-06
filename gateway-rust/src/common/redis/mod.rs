use redis::AsyncCommands;
use serde::Serialize;
use tracing::error;

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

    pub async fn get_pubsub(&self) -> anyhow::Result<redis::aio::PubSub> {
        self.client.get_async_pubsub().await.map_err(|e| {
            error!("Failed to get Redis pubsub: {}", e);
            anyhow::anyhow!(e)
        })
    }

    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.set_ex(key, value, seconds).await?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.get_connection().await?;
        let val: Option<String> = conn.get(key).await?;
        Ok(val)
    }

    pub async fn del(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let _: () = conn.del(key).await?;
        Ok(())
    }
}
