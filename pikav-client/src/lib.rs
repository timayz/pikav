use actix_rt::time::{interval_at, sleep, Instant};
use error::ClientError;
use parking_lot::RwLock;
use serde::Deserialize;
use serde_json::Map;
use std::{collections::HashMap, sync::Arc, time::Duration};
use timada::{pikav_client::PikavClient, PublishRequest, SubscribeReply, UnsubscribeReply};
use tonic::transport::Channel;
use tracing::error;
use url::Url;

pub use timada::{value::Kind, Event, ListValue, SubscribeRequest, UnsubscribeRequest, Value};
pub use tonic::Status;

mod error;

pub mod timada {
    tonic::include_proto!("timada");
}

impl Into<serde_json::Value> for Value {
    fn into(self) -> serde_json::Value {
        match self.kind {
            Some(kind) => match kind {
                Kind::DoubleValue(value) => serde_json::value::Number::from_f64(value)
                    .map(|n| serde_json::Value::Number(n))
                    .unwrap_or(serde_json::Value::Null),
                Kind::FloatValue(value) => serde_json::value::Number::from_f64(value.into())
                    .map(|n| serde_json::Value::Number(n))
                    .unwrap_or(serde_json::Value::Null),
                Kind::Int32Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Int64Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Uint32Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Uint64Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Sint32Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Sint64Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Fixed32Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Fixed64Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Sfixed32Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::Sfixed64Value(value) => {
                    serde_json::Value::Number(serde_json::value::Number::from(value))
                }
                Kind::BoolValue(value) => serde_json::Value::Bool(value),
                Kind::StringValue(value) => serde_json::Value::String(value),
                Kind::ListValue(value) => {
                    serde_json::Value::Array(value.values.into_iter().map(|v| v.into()).collect())
                }
                Kind::StructValue(value) => {
                    let mut fields = Map::new();

                    for (key, value) in &value.fields {
                        fields.insert(key.to_owned(), value.clone().into());
                    }

                    serde_json::Value::Object(fields)
                }
            },
            None => serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientOptions<N: Into<String>> {
    pub url: String,
    pub namespace: N,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClientInstanceOptions {
    pub url: String,
    pub namespace: Option<String>,
}

#[derive(Clone)]
pub struct Client {
    channel: Channel,
    queue: Arc<RwLock<Vec<Event>>>,
    namespace: Option<String>,
    pub same_region: bool,
}

impl Client {
    pub fn from_vec<T: Into<String>>(values: Vec<T>) -> Result<Vec<Self>, Vec<ClientError>> {
        let mut clients = Vec::new();
        let mut errors = Vec::new();

        for value in values {
            match Self::new_instance(ClientInstanceOptions {
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

    pub fn new<N: Into<String>>(options: ClientOptions<N>) -> Result<Self, ClientError> {
        Self::new_instance(ClientInstanceOptions {
            url: options.url,
            namespace: Some(options.namespace.into()),
        })
    }

    fn new_instance(options: ClientInstanceOptions) -> Result<Self, ClientError> {
        let parsed_url =
            Url::parse(options.url.as_str()).map_err(|e| ClientError::Unknown(e.to_string()))?;

        let query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

        let channel = Channel::from_shared(options.url.to_owned())
            .map_err(|e| ClientError::Unknown(e.to_string()))?
            .connect_lazy();

        let same_region = query
            .get("same_region")
            .map(|r| r == "true")
            .unwrap_or(false);

        let client = Self {
            channel,
            queue: Arc::new(RwLock::new(Vec::new())),
            namespace: options.namespace,
            same_region,
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
                        let mut event = event.clone();

                        if let Some(namespace) = &me.namespace {
                            event.topic = format!("{}/{}", namespace, event.topic)
                        }

                        events.push(event.clone());
                    }

                    events
                };

                if events.is_empty() {
                    continue;
                }

                let event_size = events.len();
                let mut client = PikavClient::new(me.channel.clone());

                let request = tonic::Request::new(PublishRequest {
                    propagate: me.namespace.is_some(),
                    events,
                });

                if let Err(e) = client.publish(request).await {
                    error!("{e}");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }

                {
                    let mut queue = me.queue.write();
                    queue.drain(0..event_size);
                }
            }
        });
    }

    pub fn publish(&self, events: Vec<Event>) {
        let mut queue = self.queue.write();
        queue.extend(events);
    }

    pub async fn subscribe(
        &self,
        message: SubscribeRequest,
    ) -> Result<tonic::Response<SubscribeReply>, Status> {
        let mut client = PikavClient::new(self.channel.clone());

        let request = tonic::Request::new(message);

        client.subscribe(request).await
    }

    pub async fn unsubscribe(
        &self,
        message: UnsubscribeRequest,
    ) -> Result<tonic::Response<UnsubscribeReply>, Status> {
        let mut client = PikavClient::new(self.channel.clone());

        let request = tonic::Request::new(message);

        client.unsubscribe(request).await
    }
}
