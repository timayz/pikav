use anyhow::Result;
use futures::Future;
use gloo_net::http::Headers;
use pikav::Event;
use serde_json::Value;

#[derive(Clone)]
pub struct Client;

impl Client {
    pub fn new(_endpoint: impl Into<String>) -> Self {
        Self {}
    }

    pub fn run(self) -> Result<Self> {
        Ok(self)
    }

    pub fn endpoint(self, _v: impl Into<String>) -> Self {
        self
    }

    pub fn namespace(self, _v: impl Into<String>) -> Self {
        self
    }

    pub fn close(&self) {}

    pub fn get_headers<Fu>(self, _cb: impl Fn() -> Fu + 'static) -> Self
    where
        Fu: Future<Output = Result<Headers>> + 'static,
    {
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
