use std::{cell::RefCell, pin::Pin, rc::Rc};
use anyhow::Result;
use futures::Future;
use gloo_net::http::Headers;
use pikav::Event;
use serde_json::Value;

#[derive(Clone)]
pub struct Client {
    endpoint: String,
    namespace: String,
    get_headers:
        Rc<RefCell<Option<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Headers>>>>>>>>,
}

impl Client {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();

        Self {
            get_headers: Rc::default(),
            endpoint,
            namespace: "_".to_owned(),
        }
    }

    #[cfg(not(feature = "hydrate"))]
    pub fn run(self) -> Result<Self> {
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

    pub fn close(&self) {}

    pub fn get_headers<Fu>(self, cb: impl Fn() -> Fu + 'static) -> Self
    where
        Fu: Future<Output = Result<Headers>> + 'static,
    {
        let get_headers = self.get_headers.clone();
        *get_headers.borrow_mut() = Some(Box::new(move || Box::pin(cb())));

        self
    }

    pub fn subscribe<Fu>(
        &self,
        _filter: impl Into<String>,
        _listener: impl Fn(Event<Value, Value>) -> Fu + 'static,
    ) -> impl FnOnce()
    where
        Fu: Future<Output = ()> + 'static + Send,
    {
        move || {}
    }
}
