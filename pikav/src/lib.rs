use nanoid::nanoid;
use parking_lot::RwLock;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::mpsc::{channel, error::TrySendError, Sender},
    time::{interval_at, Instant},
};
use topic::{TopicFilter, TopicName};

pub use tokio::sync::mpsc::Receiver;

pub mod topic;

#[derive(Debug)]
pub enum Error {
    SessionNotFound,
}

struct Ignore;

impl From<Ignore> for Metadata<bool> {
    fn from(_v: Ignore) -> Self {
        Metadata::Ignore
    }
}

#[derive(Debug)]
enum Metadata<T: Serialize> {
    Ignore,
    Use(T),
}

impl<T: Serialize> From<Option<T>> for Metadata<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => Metadata::Use(v),
            None => Metadata::Ignore,
        }
    }
}

impl<T: Serialize> Serialize for Metadata<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Metadata::Ignore => serializer.serialize_none(),
            Metadata::Use(data) => data.serialize(serializer),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Event<D: Serialize, M: Serialize> {
    pub topic: TopicName,
    pub name: String,
    pub data: D,
    pub metadata: M,
}

impl<D: Serialize, M: Serialize> Event<D, Metadata<M>> {
    pub fn new<N: Into<String>, T: Into<Metadata<M>>>(
        topic: TopicName,
        name: N,
        data: D,
        metadata: T,
    ) -> Self {
        Event {
            topic,
            name: name.into(),
            data,
            metadata: metadata.into(),
        }
    }
}

#[derive(Debug)]
pub struct Client<T: From<String> + Clone + Debug + Sync + Send + 'static> {
    user_id: RwLock<Option<String>>,
    sender: Sender<T>,
    filters: RwLock<Vec<TopicFilter>>,
}

impl<T: From<String> + Clone + Debug + Sync + Send + 'static> Client<T> {
    pub fn new(sender: Sender<T>) -> Self {
        Self {
            sender,
            filters: RwLock::new(Vec::new()),
            user_id: RwLock::new(None),
        }
    }

    pub fn update_user_id(&self, id: String) -> bool {
        {
            let current_id = self.user_id.read();

            if current_id.eq(&Some(id.to_owned())) {
                return false;
            }
        }

        let mut current_id = self.user_id.write();
        *current_id = Some(id);

        let mut filters = self.filters.write();
        filters.clear();

        true
    }

    pub fn is_stale(&self) -> bool {
        self.sender
            .try_send("data: ping\n\n".to_owned().into())
            .is_err()
    }

    pub fn insert(&self, filter: TopicFilter) -> bool {
        {
            let filters = self.filters.read();

            if filters.iter().any(|f| f == &filter) {
                return false;
            }
        }

        let mut filters = self.filters.write();
        filters.push(filter);

        true
    }

    pub fn remove(&self, filter: TopicFilter) -> bool {
        {
            let filters = self.filters.read();

            if filters.iter().any(|f| f == &filter) {
                return filters.is_empty();
            }
        }

        let mut filters = self.filters.write();
        filters.retain(|f| f != &filter);

        filters.is_empty()
    }

    pub fn send<D: Serialize, M: Serialize>(
        &self,
        event: &Event<D, M>,
    ) -> Result<(), TrySendError<T>> {
        let message = serde_json::to_string(event).unwrap();

        self.sender
            .clone()
            .try_send(["data: ", message.as_ref(), "\n\n"].concat().into())
    }

    pub fn filter_send<D: Serialize, M: Serialize>(
        &self,
        event: &Event<D, M>,
    ) -> Result<(), TrySendError<T>> {
        let filters = self.filters.read();

        for filter in filters.iter() {
            if filter.get_matcher().is_match(&event.topic) {
                self.send(event)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PubEvent<D: Serialize, M: Serialize> {
    pub user_id: String,
    pub event: Event<D, M>,
}

pub struct SubscribeOptions {
    pub filter: TopicFilter,
    pub user_id: String,
    pub client_id: String,
}

#[derive(Clone)]
pub struct Pikav<T: From<String> + Clone + Debug + Sync + Send + 'static> {
    clients: Arc<RwLock<HashMap<String, Client<T>>>>,
    user_clients: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl<T: From<String> + Clone + Debug + Sync + Send + 'static> Pikav<T> {
    pub fn new() -> Self {
        let me = Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            user_clients: Arc::new(RwLock::new(HashMap::new())),
        };

        Self::spawn_ping(me.clone());

        me
    }

    fn spawn_ping(me: Self) {
        tokio::spawn(async move {
            let mut interval = interval_at(Instant::now(), Duration::from_secs(10));

            loop {
                interval.tick().await;
                me.remove_stale_clients();
            }
        });
    }

    fn remove_stale_clients(&self) {
        let ids = {
            let clients = self.clients.read();
            let mut ids = Vec::new();

            for (id, c) in clients.iter() {
                if c.is_stale() {
                    ids.push((id.to_owned(), c.user_id.read().to_owned()));
                }
            }

            ids
        };

        let mut clients = self.clients.write();

        for (client_id, user_id) in ids {
            clients.remove(client_id.as_str());

            if let Some(user_id) = user_id {
                self.remove_user_client(&user_id, &client_id);
            }
        }
    }

    fn remove_user_client(&self, user_id: &str, client_id: &str) {
        let mut user_clients = self.user_clients.write();
        let is_empty = user_clients
            .get_mut(user_id)
            .map(|ids| {
                ids.remove(client_id);

                ids.is_empty()
            })
            .unwrap_or(false);

        if is_empty {
            user_clients.remove(user_id);
        }
    }

    pub fn new_client(&self) -> Option<Receiver<T>> {
        let id = nanoid!();
        let (tx, rx) = channel::<T>(100);

        let c = Client::new(tx);

        let sent = c
            .send(&Event::new(
                TopicName::new("$SYS/session").unwrap(),
                "Created",
                id.as_str(),
                Ignore,
            ))
            .is_ok();

        if !sent {
            return None;
        }

        let mut w = self.clients.write();
        w.insert(id, c);

        Some(rx)
    }

    pub fn subscribe(&self, options: SubscribeOptions) -> Result<(), Error> {
        let clients = self.clients.read();
        let client = match clients.get(&options.client_id) {
            Some(c) => c,
            None => return Err(Error::SessionNotFound),
        };

        if client.update_user_id(options.user_id.to_owned()) {
            self.remove_user_client(&options.user_id, &options.client_id);
        }

        if !client.insert(options.filter) {
            return Ok(());
        }

        let mut user_clients = self.user_clients.write();
        user_clients
            .entry(options.user_id)
            .or_insert(HashSet::new())
            .insert(options.client_id);

        Ok(())
    }

    pub fn unsubscribe(&self, options: SubscribeOptions) -> Result<(), Error> {
        let clients = self.clients.read();
        let client = match clients.get(&options.client_id) {
            Some(c) => c,
            None => return Err(Error::SessionNotFound),
        };

        if !client.remove(options.filter) {
            return Ok(());
        }

        self.remove_user_client(&options.user_id, &options.client_id);

        Ok(())
    }

    pub fn publish<D: Serialize, M: Serialize>(&self, events: Vec<&PubEvent<D, M>>) {
        let user_clients = self.user_clients.read();
        let clients = self.clients.read();

        for event in events {
            let ids = match user_clients.get(&event.user_id) {
                Some(clients) => clients,
                None => continue,
            };

            for id in ids {
                if let Some(client) = clients.get(id) {
                    let _ = client.filter_send(&event.event);
                }
            }
        }
    }
}
