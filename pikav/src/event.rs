use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::topic::TopicName;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<D, M> {
    pub topic: TopicName,
    pub name: String,
    pub data: D,
    pub metadata: Option<M>,
}

impl<D, M> Event<D, M> {
    pub fn with_metadata(topic: TopicName, name: impl Into<String>, data: D) -> Self {
        Event {
            topic,
            name: name.into(),
            data,
            metadata: None::<M>,
        }
    }

    pub fn metadata(mut self, value: M) -> Self {
        self.metadata = Some(value);

        self
    }
}

impl<D> Event<D, bool> {
    pub fn new(topic: TopicName, name: impl Into<String>, data: D) -> Self {
        Event {
            topic,
            name: name.into(),
            data,
            metadata: None::<bool>,
        }
    }
}
