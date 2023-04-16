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
        let endpoint = self.endpoint.to_owned();
        let id = self.id.clone();
        let listeners = self.listeners.clone();
        let get_headers = self.get_headers.clone();

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

                            let url = Self::get_url(endpoint.to_owned(), "subscribe", filter);
                            let headers = match get_headers.borrow().as_ref() {
                                Some(f) => match f().await {
                                    Ok(h) => Some(h),
                                    Err(e) => {
                                        error!("{e}");
                                        None
                                    }
                                },
                                None => None,
                            };
                            if let Err(e) = Self::fetch(client_id, &url, headers).await {
                                error!("{e}");
                                continue;
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

        if let (Some(client_id), 1) = (self.id.borrow().to_owned(), total_filters) {
            let filter = filter.clone();
            let endpoint = self.endpoint.clone();
            let get_headers = self.get_headers.clone();

            spawn_local(async move {
                let url = Self::get_url(endpoint.to_owned(), "subscribe", &filter);
                let headers = match get_headers.borrow().as_ref() {
                    Some(f) => match f().await {
                        Ok(h) => Some(h),
                        Err(e) => {
                            error!("{e}");
                            None
                        }
                    },
                    None => None,
                };

                if let Err(e) = Self::fetch(&client_id, &url, headers).await {
                    error!("{e}");
                }
            });
        }

        let filter = filter.clone();
        let endpoint = self.endpoint.clone();
        let client_id = self.id.clone();
        let get_headers = self.get_headers.clone();

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
                    let url = Self::get_url(endpoint.to_owned(), "unsubscribe", &filter);
                    let headers = match get_headers.borrow().as_ref() {
                        Some(f) => match f().await {
                            Ok(h) => Some(h),
                            Err(e) => {
                                error!("{e}");
                                None
                            }
                        },
                        None => None,
                    };

                    if let Err(e) = Self::fetch(&client_id, &url, headers).await {
                        error!("{e}");
                    }
                });
            }
        }
    }

    fn get_url(
        endpoint: impl Into<String>,
        action: impl Into<String>,
        filter: &TopicFilter,
    ) -> String {
        format!(
            "{}/{}/{}",
            endpoint.into(),
            action.into(),
            filter.to_string()
        )
    }

    async fn fetch(client_id: &str, url: &str, headers: Option<Headers>) -> Result<Response> {
        let mut req = Request::put(url);

        if let Some(headers) = headers {
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
