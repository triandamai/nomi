use crate::feature::conversation::internal_model::InboundMessage;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, error};

pub async fn bridge_inbound(message: InboundMessage) -> anyhow::Result<()> {
    let client = Client::new();
    let gateway_url = std::env::var("GATEWAY_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let url = format!("{}/api/internal/inbound", gateway_url);

    info!("Bridging message to Brain: {:?}", message);

    let res = client.post(url)
        .json(&message)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;

    if !res.status().is_success() {
        error!("Failed to bridge message: {}", res.status());
    }

    Ok(())
}
