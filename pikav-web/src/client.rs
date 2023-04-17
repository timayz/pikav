use std::{
    cell::RefCell,
    collections::HashSet,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use futures::{future::BoxFuture, Future, StreamExt};
use gloo_net::{
    eventsource::futures::EventSource,
    http::{Headers, Request, Response},
};
use log::error;
use pikav::{topic::TopicFilter, Event};
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct Client {
    id: Rc<RefCell<Option<String>>>,
    source_url: String,
    source: Rc<RefCell<Option<EventSource>>>,
    endpoint: String,
    namespace: String,
    next_listener_id: Rc<AtomicUsize>,
    get_headers: Rc<RefCell<Option<Box<dyn Fn() -> BoxFuture<'static, Result<Headers>>>>>>,
    listeners: Rc<
        RefCell<
            Vec<(
                usize,
                TopicFilter,
                Box<dyn Fn(Event<Value, Value>) -> BoxFuture<'static, ()>>,
            )>,
        >,
    >,
}

impl Client {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();

        Self {
            id: Rc::default(),
            get_headers: Rc::default(),
            next_listener_id: Rc::default(),
            listeners: Rc::default(),
            source: Rc::default(),
            source_url: format!("{endpoint}/events"),
            endpoint,
            namespace: "_".to_owned(),
        }
    }

    pub fn run(self) -> Result<Self> {
        let mut source = gloo_net::eventsource::futures::EventSource::new(&self.source_url)?;
        let mut stream = source.subscribe("message")?;
        *self.source.borrow_mut() = Some(source);
        let id = self.id.clone();
        let listeners = self.listeners.clone();
        let fetcher = Fetcher::from(&self);

        spawn_local(async move {
            while let Some(Ok((_, msg))) = stream.next().await {
                if msg.data().as_string() == Some("ping".to_owned()) {
                    continue;
                }

                let data = match msg.data().as_string() {
                    Some(data) => data,
                    _ => {
                        error!("invalid type: {:?}", msg.data());
                        continue;
                    }
                };

                let event = match serde_json::from_str::<Event<Value, Value>>(&data) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("invalid type: {:?}", e);
                        continue;
                    }
                };

                if matches!(
                    (event.topic.as_ref(), event.name.as_ref()),
                    ("$SYS/session", "Created")
                ) {
                    *id.borrow_mut() = event.data.as_str().map(|v| v.to_owned());

                    let mut subscribed = HashSet::new();

                    if let Some(client_id) = event.data.as_str() {
                        for (_, filter, _) in listeners.borrow().iter() {
                            if subscribed.contains(filter) {
                                continue;
                            }

                            if let Err(e) = fetcher.fetch(&client_id, "subscribe", &filter).await {
                                error!("{e}");
                            }

                            subscribed.insert(filter);
                        }
                    }
                }

                for (_, filter, listener) in listeners.borrow().iter() {
                    if filter.get_matcher().is_match(&event.topic) {
                        listener(event.clone()).await;
                    }
                }
            }
        });

        Ok(self)
    }

    pub fn endpoint(mut self, v: impl Into<String>) -> Self {
        self.endpoint = v.into();

        self
    }

    pub fn namespace(mut self, v: impl Into<String>) -> Self {
        self.namespace = v.into();

        self
    }

    pub fn close(&self) {
        if let Some(source) = self.source.borrow().as_ref() {
            source.clone().close();
        }
    }

    pub fn get_headers<Fu>(self, cb: impl Fn() -> Fu + 'static) -> Self
    where
        Fu: Future<Output = Result<Headers>> + 'static + Send,
    {
        let get_headers = self.get_headers.clone();
        *get_headers.borrow_mut() = Some(Box::new(move || Box::pin(cb())));

        self
    }

    pub fn subscribe<Fu>(
        &self,
        filter: impl Into<String>,
        listener: impl Fn(Event<Value, Value>) -> Fu + 'static,
    ) -> impl FnOnce()
    where
        Fu: Future<Output = ()> + 'static + Send,
    {
        let filter = TopicFilter::new(filter).unwrap_or_else(|e| panic!("{e}"));
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        let listeners = self.listeners.clone();

        listeners
            .borrow_mut()
            .push((id, filter.clone(), Box::new(move |e| Box::pin(listener(e)))));

        let total_filters = listeners
            .borrow()
            .iter()
            .filter(|(_, f, _)| f == &filter)
            .count();

        let fetcher = Fetcher::from(self);

        if let (Some(client_id), 1) = (self.id.borrow().to_owned(), total_filters) {
            let filter = filter.clone();
            let fetcher = fetcher.clone();

            spawn_local(async move {
                if let Err(e) = fetcher.fetch(&client_id, "subscribe", &filter).await {
                    error!("{e}");
                }
            });
        }

        let filter = filter.clone();
        let client_id = self.id.clone();

        move || {
            listeners.borrow_mut().retain(|l| l.0 != id);

            let total_filters = listeners
                .borrow()
                .iter()
                .filter(|(_, f, _)| f == &filter)
                .count();

            if total_filters > 0 {
                return;
            }

            if let Some(client_id) = client_id.borrow().to_owned() {
                spawn_local(async move {
                    if let Err(e) = fetcher.fetch(&client_id, "unsubscribe", &filter).await {
                        error!("{e}");
                    }
                });
            }
        }
    }
}

#[derive(Clone)]
struct Fetcher {
    namespace: String,
    endpoint: String,
    get_headers: Rc<RefCell<Option<Box<dyn Fn() -> BoxFuture<'static, Result<Headers>>>>>>,
}

impl Fetcher {
    pub async fn fetch(
        &self,
        client_id: &str,
        action: impl Into<String>,
        filter: &TopicFilter,
    ) -> Result<Response> {
        let filter = filter.to_string();
        let mut req = Request::put(&format!(
            "{}/{}/{}/{}",
            self.endpoint,
            action.into(),
            self.namespace,
            filter
        ));

        if let Some(get_header) = self.get_headers.borrow().as_ref() {
            let headers = get_header().await?;
            req = req.headers(headers);
        }

        let res = req
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("X-Pikav-Client-ID", client_id)
            .send()
            .await?;

        Ok(res)
    }
}

impl From<&Client> for Fetcher {
    fn from(value: &Client) -> Self {
        Self {
            namespace: value.namespace.to_owned(),
            endpoint: value.endpoint.to_owned(),
            get_headers: value.get_headers.clone(),
        }
    }
}
