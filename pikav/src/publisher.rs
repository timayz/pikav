use futures::StreamExt;
use glob_match::glob_match;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{
        mpsc::{channel, error::TrySendError, Sender},
        RwLock,
    },
    time::{interval_at, Instant},
};

pub use tokio::sync::mpsc::Receiver;

use crate::event::{Event, SimpleEvent};

#[derive(Debug)]
pub enum Error {
    SessionNotFound,
}

#[derive(Debug)]
pub struct Client<T: From<String> + Clone + Debug + Sync + Send + 'static> {
    user_id: RwLock<Option<String>>,
    sender: Sender<T>,
    filters: RwLock<Vec<String>>,
}

impl<T: From<String> + Clone + Debug + Sync + Send + 'static> Client<T> {
    pub fn new(sender: Sender<T>) -> Self {
        Self {
            sender,
            filters: RwLock::new(Vec::new()),
            user_id: RwLock::new(None),
        }
    }

    pub async fn update_user_id(&self, id: String) -> bool {
        {
            let current_id = self.user_id.read().await;

            if current_id.eq(&Some(id.to_owned())) {
                return false;
            }
        }

        let mut current_id = self.user_id.write().await;
        *current_id = Some(id);

        let mut filters = self.filters.write().await;
        filters.clear();

        true
    }

    pub fn is_stale(&self) -> bool {
        self.sender
            .try_send("data: ping\n\n".to_owned().into())
            .is_err()
    }

    pub async fn insert(&self, filter: String) -> bool {
        {
            let filters = self.filters.read().await;

            if filters.iter().any(|f| f == &filter) {
                return false;
            }
        }

        let mut filters = self.filters.write().await;
        filters.push(filter);

        true
    }

    pub async fn remove(&self, filter: String) -> bool {
        {
            let filters = self.filters.read().await;

            if filters.iter().any(|f| f == &filter) {
                return filters.is_empty();
            }
        }

        let mut filters = self.filters.write().await;
        filters.retain(|f| f != &filter);

        filters.is_empty()
    }

    pub fn send_event_session_id(&self, id: impl Into<String>) -> Result<(), TrySendError<T>> {
        self.send_event(Event::new("$SYS/session", "Created", id.into()))
    }

    pub fn send_event<D: Serialize, M: Serialize>(
        &self,
        event: Event<D, M>,
    ) -> Result<(), TrySendError<T>> {
        let data = serde_json::to_string(&event).unwrap();

        self.send(SimpleEvent {
            topic: event.topic,
            event: "message".to_owned(),
            data,
        })
    }

    pub fn send(&self, event: SimpleEvent) -> Result<(), TrySendError<T>> {
        self.sender.clone().try_send(
            [
                "event: ",
                event.event.as_ref(),
                "\n",
                "data: ",
                event.data.as_ref(),
                "\n\n",
            ]
            .concat()
            .into(),
        )
    }

    pub async fn filter_send_event<D: Serialize, M: Serialize>(
        &self,
        event: Event<D, M>,
    ) -> Result<(), TrySendError<T>> {
        let rw_filters = self.filters.read().await;

        let filters = rw_filters
            .iter()
            .filter_map(|filter| match glob_match(filter, &event.topic) {
                true => Some(filter.to_owned()),
                false => None,
            })
            .collect::<Vec<_>>();

        if !filters.is_empty() {
            self.send_event(event.filters(filters))?;
        }

        Ok(())
    }

    pub async fn filter_send(&self, event: SimpleEvent) -> Result<(), TrySendError<T>> {
        let rw_filters = self.filters.read().await;

        let filters = rw_filters
            .iter()
            .filter_map(|filter| match glob_match(filter, &event.topic) {
                true => Some(filter.to_owned()),
                false => None,
            })
            .collect::<Vec<_>>();

        if !filters.is_empty() {
            self.send(event)?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<E> {
    pub event: E,
    pub user_id: String,
}

#[derive(Clone)]
pub struct Publisher<T: From<String> + Clone + Debug + Sync + Send + 'static> {
    clients: Arc<RwLock<HashMap<String, Client<T>>>>,
    user_clients: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl<T: From<String> + Clone + Debug + Sync + Send + 'static> Publisher<T> {
    pub fn start() -> Self {
        let publisher = Self::default();

        tokio::spawn({
            let publisher = publisher.clone();

            async move {
                let mut interval = interval_at(Instant::now(), Duration::from_secs(10));

                loop {
                    interval.tick().await;

                    publisher.remove_stale_clients().await;
                }
            }
        });

        publisher
    }

    async fn remove_stale_clients(&self) {
        let ids = {
            let clients = self.clients.read().await;
            let mut ids = Vec::new();

            for (id, c) in clients.iter() {
                if c.is_stale() {
                    ids.push((id.to_owned(), c.user_id.read().await.to_owned()));
                }
            }

            ids
        };

        let mut clients = self.clients.write().await;

        for (client_id, user_id) in ids {
            clients.remove(client_id.as_str());

            if let Some(user_id) = user_id {
                self.remove_user_client(&user_id, &client_id).await;
            }
        }
    }

    async fn remove_user_client(&self, user_id: &str, client_id: &str) {
        let mut user_clients = self.user_clients.write().await;
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

    pub async fn create_client(&self, send_id: bool) -> Option<Receiver<T>> {
        let id = nanoid!();
        let (tx, rx) = channel::<T>(100);
        let client = Client::new(tx);

        if send_id && client.send_event_session_id(&id).is_err() {
            return None;
        }

        let mut w = self.clients.write().await;

        w.insert(id, client);

        Some(rx)
    }

    pub async fn subscribe(
        &self,
        filter: String,
        user_id: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Result<(), Error> {
        let user_id = user_id.into();
        let client_id = client_id.into();
        let clients = self.clients.read().await;

        let client = match clients.get(&client_id) {
            Some(c) => c,
            None => return Err(Error::SessionNotFound),
        };

        if client.update_user_id(user_id.to_owned()).await {
            self.remove_user_client(&user_id, &client_id).await;
        }

        if !client.insert(filter).await {
            return Ok(());
        }

        let mut user_clients = self.user_clients.write().await;
        user_clients
            .entry(user_id)
            .or_insert_with(HashSet::new)
            .insert(client_id);

        Ok(())
    }

    pub async fn unsubscribe(
        &self,
        filter: String,
        user_id: impl Into<String>,
        client_id: impl Into<String>,
    ) -> Result<(), Error> {
        let user_id = user_id.into();
        let client_id = client_id.into();
        let clients = self.clients.read().await;

        let client = match clients.get(&client_id) {
            Some(c) => c,
            None => return Err(Error::SessionNotFound),
        };

        if !client.remove(filter).await {
            return Ok(());
        }

        self.remove_user_client(&user_id, &client_id).await;

        Ok(())
    }

    pub async fn publish(&self, events: Vec<&Message<SimpleEvent>>) {
        let user_clients = self.user_clients.read().await;
        let clients = self.clients.read().await;
        let mut futures = Vec::new();

        for event in events {
            let ids = match user_clients.get(&event.user_id) {
                Some(clients) => clients,
                None => continue,
            };

            for id in ids {
                if let Some(client) = clients.get(id) {
                    futures.push(client.filter_send(event.event.clone()));
                }
            }
        }

        let stream = futures::stream::iter(futures).buffer_unordered(50);
        let _ = stream.collect::<Vec<_>>().await;
    }

    pub async fn publish_events<D: Serialize + Clone, M: Serialize + Clone>(
        &self,
        events: Vec<&Message<Event<D, M>>>,
    ) {
        let user_clients = self.user_clients.read().await;
        let clients = self.clients.read().await;
        let mut futures = Vec::new();

        for event in events {
            let ids = match user_clients.get(&event.user_id) {
                Some(clients) => clients,
                None => continue,
            };

            for id in ids {
                if let Some(client) = clients.get(id) {
                    futures.push(client.filter_send_event(event.event.clone()));
                }
            }
        }

        let stream = futures::stream::iter(futures).buffer_unordered(50);
        let _ = stream.collect::<Vec<_>>().await;
    }
}

impl<T: From<String> + Clone + Debug + Sync + Send + 'static> Default for Publisher<T> {
    fn default() -> Self {
        Self {
            clients: Arc::default(),
            user_clients: Arc::default(),
        }
    }
}
