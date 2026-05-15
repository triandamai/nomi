use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use futures::Stream;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;
use crate::common::api_response::ApiResponse;
use crate::common::sse::sse_builder::SseBuilder;

#[derive(Debug)]
pub struct SseBroadcaster {
    inner: Mutex<SseBroadcasterInner>,
}

#[derive(Debug, Clone, Default)]
pub struct SseBroadcasterInner {
    /**
     *   each user subscribe to channel<br>
     *   each user device subscribe to child of channel <br>
     *   example:<br>
     *   [<br>
     *        "+6281235623":[<br>
     *           "device1:"event",<br>
     *            "device2:"event"<br>
     *        ]<br>
     *    ]
     */
    clients: HashMap<String, HashMap<String, Sender<Event>>>,
}

impl SseBroadcaster {
    pub fn create() -> Arc<Self> {
        let this = Arc::new(SseBroadcaster {
            inner: Mutex::new(SseBroadcasterInner::default()),
        });
        SseBroadcaster::spawn_ping(Arc::clone(&this));
        this
    }

    /// Pings clients every 10 seconds to see if they are alive and remove them from the broadcast
    /// list if not.'
    fn spawn_ping(this: Arc<Self>) {
        spawn(async move {
            let mut interval = interval(Duration::from_secs(15));
            loop {
                interval.tick().await;
                this.remove_stale_client().await;
            }
        });
    }

    /// Removes ALL non-responsive clients from broadcast list.
    async fn remove_stale_client(&self) {
        let clients_snapshot = {
            let inner = self.inner.lock().unwrap();
            inner.clients.clone()
        };

        let mut stale_clients = Vec::new();

        for (user_id, devices) in clients_snapshot {
            for (device_id, client) in devices {
                let ping = Event::default()
                    .event("ping")
                    .json_data("keep-alive")
                    .unwrap();

                // If the channel is closed, mark for removal. 
                // If it's full, we skip this ping but keep the client.
                match client.try_send(ping) {
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                        stale_clients.push((user_id.clone(), device_id.clone()));
                    }
                    _ => {}
                }
            }
        }

        if !stale_clients.is_empty() {
            let mut inner = self.inner.lock().unwrap();
            for (user_id, device_id) in stale_clients {
                if let Some(devices) = inner.clients.get_mut(&user_id) {
                    devices.remove(&device_id);
                    if devices.is_empty() {
                        inner.clients.remove(&user_id);
                    }
                }
            }
        }
    }

    pub async fn new_client<'a>(
        &self,
        user_id: String,
        device_id: String,
        model_info: crate::common::agent::agent_model::ModelInfo,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>> + use<'a>> {
        let (tx, rx) = mpsc::channel(100);

        // Send initial connected event
        let event = Event::default()
            .event("connected")
            .json_data(ApiResponse::ok("CONNECTED".to_string(), "Success"))
            .unwrap();
        let _ = tx.send(event).await;

        // Task 3: Send initial metadata event
        let metadata = Event::default()
            .event("metadata")
            .json_data(model_info)
            .unwrap();
        let _ = tx.send(metadata).await;

        let stream = ReceiverStream::<Event>::new(rx).map(|res| Ok(res));

        let mut subs = match self.inner.lock().unwrap().clients.get(&user_id) {
            None => HashMap::new(),
            Some(client) => client.clone(),
        };

        subs.insert(device_id.clone(), tx.clone());
        self.inner
            .lock()
            .unwrap()
            .clients
            .insert(user_id.clone(), subs);

        Sse::new(stream).keep_alive(KeepAlive::default())
    }

    pub async fn reject_client(&self) -> Result<String, String> {
        let (tx, _) = mpsc::channel(10);

        let event = Event::default()
            .event("connection")
            .json_data(ApiResponse::ok(
                "Rejected".to_string(),
                "Failed to subscribe",
            ))
            .unwrap();

        let _ = tx.send(event).await;
        tx.closed().await;

        Ok("Ok".to_string())
    }

    pub async fn send<T: serde::Serialize>(&self, builder: SseBuilder<T>) {
        let target = builder.get_target();
        if target.is_broadcast() {
            self.broadcast(target.even_name(), &builder.data).await;
        } else {
            if target.is_to_device() {
                for user in target.user_id() {
                    self.send_to_user_device(
                            user,
                            target.device_id(),
                            target.even_name(),
                            &builder.data,
                        )
                        .await;
                }
            } else {
                for user in target.user_id() {
                    self.send_to_user(user, target.even_name(), &builder.data)
                        .await;
                }
            }
        }
    }

    async fn broadcast<T: serde::Serialize>(&self, event_name: &String, data: &T) {
        let event = Event::default()
            .event(event_name)
            .json_data(data)
            .unwrap();

        let mut closed_clients = Vec::new();

        {
            let inner = self.inner.lock().unwrap();
            for (user_id, devices) in &inner.clients {
                for (device_id, client) in devices {
                    match client.try_send(event.clone()) {
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::error!("SSE Channel closed for device {}", device_id);
                            closed_clients.push((user_id.clone(), device_id.clone()));
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::error!("SSE Channel FULL for device {}, dropping event {}", device_id, event_name);
                        }
                        Ok(_) => {}
                    }
                }
            }
        }

        if !closed_clients.is_empty() {
            self.remove_clients(closed_clients).await;
        }
    }

    async fn send_to_user<T: serde::Serialize>(
        &self,
        user_id: &String,
        event_name: &String,
        data: &T,
    ) {
        let event = Event::default()
            .event(event_name)
            .json_data(data)
            .unwrap();

        let mut closed_devices = Vec::new();

        {
            let inner = self.inner.lock().unwrap();
            if let Some(devices) = inner.clients.get(user_id) {
                for (device_id, client) in devices {
                    match client.try_send(event.clone()) {
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::error!("SSE Channel closed for device {}", device_id);
                            closed_devices.push(device_id.clone());
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::error!("SSE Channel FULL for device {}, dropping event {}", device_id, event_name);
                        }
                        Ok(_) => {}
                    }
                }
            }
        }

        if !closed_devices.is_empty() {
            let mut inner = self.inner.lock().unwrap();
            if let Some(devices) = inner.clients.get_mut(user_id) {
                for d_id in closed_devices {
                    devices.remove(&d_id);
                }
                if devices.is_empty() {
                    inner.clients.remove(user_id);
                }
            }
        }
    }

    async fn send_to_user_device<T: serde::Serialize>(
        &self,
        user_id: &String,
        device_id: &String,
        event_name: &String,
        data: &T,
    ) {
        let event = Event::default()
            .event(event_name)
            .json_data(data)
            .unwrap();

        let mut closed = false;

        {
            let inner = self.inner.lock().unwrap();
            if let Some(devices) = inner.clients.get(user_id) {
                if let Some(client) = devices.get(device_id) {
                    match client.try_send(event) {
                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            tracing::error!("SSE Channel closed for device {}", device_id);
                            closed = true;
                        }
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            tracing::error!("SSE Channel FULL for device {}, dropping event {}", device_id, event_name);
                        }
                        Ok(_) => {}
                    }
                }
            }
        }

        if closed {
            let mut inner = self.inner.lock().unwrap();
            if let Some(devices) = inner.clients.get_mut(user_id) {
                devices.remove(device_id);
                if devices.is_empty() {
                    inner.clients.remove(user_id);
                }
            }
        }
    }

    async fn remove_clients(&self, failed_clients: Vec<(String, String)>) {
        let mut inner = self.inner.lock().unwrap();
        for (user_id, device_id) in failed_clients {
            if let Some(devices) = inner.clients.get_mut(&user_id) {
                devices.remove(&device_id);
                if devices.is_empty() {
                    inner.clients.remove(&user_id);
                }
            }
        }
    }

    pub async fn get_list_client(&self) -> Option<HashMap<String, Vec<String>>> {
        let clients = self.inner.lock().unwrap().clients.clone();

        let mut data: HashMap<String, Vec<String>> = HashMap::new();

        let _ = clients.iter().for_each(|(key, sub)| {
            let items = sub.iter().map(|(key, _)| key.clone()).collect();
            data.insert(key.clone(), items);
        });

        Some(data)
    }
}
