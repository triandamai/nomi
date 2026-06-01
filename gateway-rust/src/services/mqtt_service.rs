use rumqttc::{MqttOptions, AsyncClient, QoS, Event, Packet, Transport};
use std::time::Duration;

pub struct MqttManager {
    pub client: AsyncClient,
}

impl MqttManager {
    pub fn init(client_id: &str, host: &str, port: u16, username: Option<&str>, password: Option<&str>) -> Self {
        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(15));
        mqttoptions.set_clean_session(false); // Retain persistent session rules

        // Add Credentials if provided
        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }

        // 💡 Use Rustls configurations (default) for SNI targeting
        // Standard for EMQX Cloud / HiveMQ Cloud / AWS IoT
        if port == 8883 || host.contains("cloud") || host.contains("pakaiarta.id") || host.contains("emqxsl.com") {
            let tls_config = rumqttc::TlsConfiguration::default();
            mqttoptions.set_transport(Transport::Tls(tls_config));
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 50);

        // Clone client to handle background auto-subscription boundaries cleanly
        let subscription_client = client.clone();

        tokio::spawn(async move {
            // Connect and listen for client inbound actions natively
            if let Err(e) = subscription_client.subscribe("nomi/conversations/+/commands", QoS::AtLeastOnce).await {
                eprintln!("🛑 EMQX Subscription failure: {:?}", e);
            }

            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::Publish(publish))) => {
                        let topic = publish.topic;
                        let payload = publish.payload;
                        println!("📥 EMQX Incoming Line: Topic='{}', Payload Length={}", topic, payload.len());
                        // Inbound mapping handler logic will safely sit here
                    }
                    Ok(_) => {
                      // println!("EVENT: {:?}", ok);
                    },
                    Err(e) => {
                        eprintln!("⚠️ EMQX Connection Loop Error: {:?}. Retrying loop link...", e);
                        tokio::time::sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        });

        Self { client }
    }

    /// Publish an event to a topic. Awaits the send directly so errors surface immediately.
    pub async fn publish_event(&self, topic: &str, payload: &str, qos: QoS) -> Result<(), rumqttc::ClientError> {
        self.client.publish(topic, qos, false, payload.as_bytes().to_vec()).await
    }

    /// Publish with retain=true so late-joining subscribers (e.g. TaskCard mounting after the
    /// initial publish) still receive the last known state from the broker.
    pub async fn publish_retained(&self, topic: &str, payload: &str, qos: QoS) -> Result<(), rumqttc::ClientError> {
        self.client.publish(topic, qos, true, payload.as_bytes().to_vec()).await
    }
}
