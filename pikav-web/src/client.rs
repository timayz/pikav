use std::{borrow::BorrowMut, collections::HashMap, sync::Arc};

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};
use gloo_net::eventsource::futures::EventSource;
use log::debug;
use wasm_bindgen_futures::spawn_local;
// use pikav::topic::TopicFilter;

#[derive(Clone, Debug)]
pub struct Client {
    id: Option<String>,
    source: EventSource,
    endpoint: String,
    namespace: Option<String>,
}

impl Client {
    pub fn new(url: impl Into<String>) -> Result<Self> {
        let url = url.into();
        let mut source = gloo_net::eventsource::futures::EventSource::new(&url)?;
        let mut stream = source.subscribe("message")?;

        spawn_local(async move {
            while let Some(Ok((event_type, msg))) = stream.next().await {
                debug!("1. {}: {:?}", event_type, msg.data().as_string());
                // debug!("yes");
            }
            debug!("EventSource Closed");
        });

        Ok(Self {
            id: None,
            endpoint: url.to_owned(),
            source,
            namespace: None,
        })
    }

    pub fn endpoint(mut self, v: impl Into<String>) -> Self {
        self.endpoint = v.into();

        self
    }

    pub fn namespace(mut self, v: impl Into<String>) -> Self {
        self.namespace = Some(v.into());

        self
    }

    pub fn close(&self) {
        self.source.clone().close();
    }
}
