use actix_rt::time::{interval_at, sleep, Instant};
use error::ClientError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use timada::{pikav_client::PikavClient, PublishRequest};
use std::{sync::Arc, time::Duration};
use tonic::transport::Channel;
use tracing::error;

mod error;

mod timada {
    tonic::include_proto!("timada");
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Event {
    pub user_id: String,
    pub topic: String,
    pub name: String,
    pub data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
}

impl Event {
    pub fn new<D: Serialize, U: Into<String>, T: Into<String>, N: Into<String>>(
        user_id: U,
        topic: T,
        name: N,
        data: D,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            user_id: user_id.into(),
            topic: topic.into(),
            name: name.into(),
            data: serde_json::to_value(data)?,
            metadata: None,
        })
    }

    pub fn new_with_metadata<
        D: Serialize,
        U: Into<String>,
        T: Into<String>,
        N: Into<String>,
        M: Serialize,
    >(
        user_id: U,
        topic: T,
        name: N,
        data: D,
        metadata: Option<M>,
    ) -> Result<Self, serde_json::Error> {
        let metadata = match metadata {
            Some(metadata) => Some(serde_json::to_value(metadata)?),
            None => None,
        };

        Ok(Self {
            user_id: user_id.into(),
            topic: topic.into(),
            name: name.into(),
            data: serde_json::to_value(data)?,
            metadata,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientOptions {
    pub url: String,
    pub namespace: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    channel: Channel,
    queue: Arc<RwLock<Vec<Event>>>,
}

impl Client {
    pub fn from_vec<T: Into<String>>(values: Vec<T>) -> Result<Vec<Self>, Vec<ClientError>> {
        let mut clients = Vec::new();
        let mut errors = Vec::new();

        for value in values {
            match Self::new(ClientOptions {
                url: value.into(),
                namespace: None,
            }) {
                Ok(client) => clients.push(client),
                Err(e) => errors.push(e),
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(clients)
    }

    pub fn new(options: ClientOptions) -> Result<Self, ClientError> {
        let channel = Channel::from_shared(options.url.to_owned())
            .map_err(|e| ClientError::Unknown(e.to_string()))?
            .connect_lazy();

        let client = Self {
            channel,
            queue: Arc::new(RwLock::new(Vec::new())),
        };

        Self::spawn_queue(client.clone());

        Ok(client)
    }

    fn spawn_queue(me: Self) {
        actix_rt::spawn(async move {
            let mut interval = interval_at(Instant::now(), Duration::from_millis(300));

            loop {
                interval.tick().await;

                let events = {
                    let queue = me.queue.read();

                    if queue.len() == 0 {
                        continue;
                    }

                    let mut events = Vec::new();

                    for event in queue.iter().take(1000) {
                        events.push(event.clone());
                    }

                    events
                };

                if events.is_empty() {
                    continue;
                }

                let mut client = PikavClient::new(me.channel.clone());

                let request = tonic::Request::new(PublishRequest {
                    propagate: true,
                    ..Default::default()
                });

                if let Err(e) = client.publish(request).await {
                    error!("{e}");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                {
                    let mut queue = me.queue.write();
                    queue.drain(0..events.len());
                }
            }
        });
    }

    // fn client(&self) -> awc::Client {
    //     awc::Client::builder()
    //         .add_default_header((
    //             "User-Agent",
    //             format!(
    //                 "Pikav/{}.{}",
    //                 env!("CARGO_PKG_VERSION_MAJOR"),
    //                 env!("CARGO_PKG_VERSION_MINOR")
    //             ),
    //         ))
    //         .add_default_header(("X-Pikav-Cluster", self.is_cluster.to_string()))
    //         .finish()
    // }

    // pub fn put<U: TryFrom<Uri> + Display>(&self, uri: &'_ U) -> ClientRequest {
    //     self.client().put(format!("{}{}", self.options.url, uri))
    // }

    pub fn publish(&self, events: Vec<Event>) {
        let mut queue = self.queue.write();
        queue.extend(events);
    }

    // pub fn is_shared(&self) -> bool {
    //     self.options.shared.unwrap_or(false)
    // }
}
