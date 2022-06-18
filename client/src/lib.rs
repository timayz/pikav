use actix_rt::time::{interval_at, Instant};
use awc::{http::Uri, ClientRequest};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::Arc, time::Duration};
use tracing::error;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ClusterOptions {
    pub url: String,
    pub namespace: Option<String>,
    pub shared: Option<bool>,
}

#[derive(Clone)]
pub struct Client {
    options: ClusterOptions,
    queue: Arc<RwLock<Vec<Event>>>,
    is_cluster: bool,
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self::build(
            ClusterOptions {
                url: options.url,
                namespace: options.namespace,
                shared: None,
            },
            false,
        )
    }

    pub fn cluster(options: ClusterOptions) -> Self {
        Self::build(options, true)
    }

    fn build(options: ClusterOptions, is_cluster: bool) -> Self {
        let me = Self {
            options,
            queue: Arc::new(RwLock::new(Vec::new())),
            is_cluster,
        };

        Self::spawn_queue(me.clone());

        me
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

                let client = me.client();

                let res = client
                    .post(format!(
                        "{}/publish/{}",
                        me.options.url.to_owned(),
                        me.options
                            .namespace
                            .to_owned()
                            .unwrap_or("_".to_lowercase())
                    ))
                    .send_json(&events)
                    .await;

                if let Err(e) = res {
                    error!("{}", e);
                }

                {
                    let mut queue = me.queue.write();
                    queue.drain(0..events.len());
                }
            }
        });
    }

    fn client(&self) -> awc::Client {
        awc::Client::builder()
            .add_default_header((
                "User-Agent",
                format!(
                    "Pikav/{}.{}",
                    env!("CARGO_PKG_VERSION_MAJOR"),
                    env!("CARGO_PKG_VERSION_MINOR")
                ),
            ))
            .add_default_header(("X-Pikav-Cluster", self.is_cluster.to_string()))
            .finish()
    }

    pub fn put<U: TryFrom<Uri> + Display>(&self, uri: &'_ U) -> ClientRequest {
        self.client().put(format!("{}{}", self.options.url, uri))
    }

    pub fn publish(&self, events: Vec<Event>) {
        let mut queue = self.queue.write();
        queue.extend(events);
    }

    pub fn is_shared(&self) -> bool {
        self.options.shared.unwrap_or(false)
    }
}
