use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimpleEvent {
    pub topic: String,
    pub event: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<D, M> {
    pub topic: String,
    pub name: String,
    pub data: D,
    pub metadata: Option<M>,
    pub filters: Option<Vec<String>>,
}

impl<D, M> Event<D, M> {
    pub fn with_metadata(topic: impl Into<String>, name: impl Into<String>, data: D) -> Self {
        Event {
            topic: topic.into(),
            name: name.into(),
            data,
            metadata: None::<M>,
            filters: None,
        }
    }

    pub fn metadata(mut self, value: M) -> Self {
        self.metadata = Some(value);

        self
    }

    pub fn filters(mut self, value: Vec<String>) -> Self {
        self.filters = Some(value);

        self
    }
}

impl<D> Event<D, bool> {
    pub fn new(topic: impl Into<String>, name: impl Into<String>, data: D) -> Self {
        Event {
            topic: topic.into(),
            name: name.into(),
            data,
            metadata: None::<bool>,
            filters: None,
        }
    }
}
