use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    thread::sleep,
    time::Duration,
};

use anyhow::Result;
use futures::{future::BoxFuture, Future, StreamExt};
use gloo_net::eventsource::futures::EventSource;
use log::error;
use pikav::{topic::TopicFilter, Event};
use serde_json::Value;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone)]
pub struct Client {
    id: Rc<RefCell<Option<String>>>,
    source: EventSource,
    endpoint: String,
    namespace: String,
    next_listener_id: Rc<AtomicUsize>,
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
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        let mut source = gloo_net::eventsource::futures::EventSource::new(&url)?;
        let mut stream = source.subscribe("message")?;
        let id = Rc::new(RefCell::new(None));
        let listeners = Rc::new(RefCell::new(Vec::new()));

        let c = Self {
            id: id.clone(),
            listeners: listeners.clone(),
            next_listener_id: Rc::default(),
            endpoint: url.to_owned(),
            namespace: "_".to_owned(),
            source,
        };

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
                    // subscribe all request
                }

                for (_, filter, listener) in listeners.borrow().iter() {
                    if filter.get_matcher().is_match(&event.topic) {
                        listener(event.clone()).await;
                    }
                }
            }
        });

        Ok(c)
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
        self.source.clone().close();
    }

    pub fn subscribe<Fu>(
        &self,
        filter: impl Into<String>,
        listener: impl Fn(Event<Value, Value>) -> Fu + 'static,
    ) -> Result<impl Fn()>
    where
        Fu: Future<Output = ()> + 'static + Send,
    {
        let filter = TopicFilter::new(filter)?;
        let id = self.next_listener_id.fetch_add(1, Ordering::Relaxed);
        let listeners = self.listeners.clone();

        listeners
            .borrow_mut()
            .push((id, filter, Box::new(move |e| Box::pin(listener(e)))));

        if let Some(id) = self.id.borrow().clone() {
            // send request to subscribe
        }

        Ok(move || {
            listeners.borrow_mut().retain(|l| l.0 != id);
            // unsubscribe request
        })
    }
}
