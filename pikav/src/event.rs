use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::topic::TopicName;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event<D, M> {
    pub topic: TopicName,
    pub name: String,
    pub data: D,
    pub metadata: Option<M>,
}

impl<D, M> Event<D, M>
where
    D: Serialize + DeserializeOwned,
    M: Serialize + DeserializeOwned,
{
    pub fn new(topic: TopicName, name: impl Into<String>, data: D, metadata: Option<M>) -> Self {
        Event {
            topic,
            name: name.into(),
            data,
            metadata,
        }
    }
}
