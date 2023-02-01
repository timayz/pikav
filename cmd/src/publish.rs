use std::{collections::HashMap, time::Duration};

use config::{Config, ConfigError, Environment, File};
use pikav_client::{timada::Struct, Client, ClientOptions, Event, Kind, Value};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PublishAddr {
    pub api: String,
    pub cluster: String,
}

#[derive(Debug, Deserialize)]
pub struct Publish {
    pub addr: PublishAddr,
}

impl Publish {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name(path))
            .add_source(File::with_name(&format!("{path}.local")).required(false))
            .add_source(Environment::with_prefix(env!("CARGO_PKG_NAME")))
            .build()?
            .try_deserialize()
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let client = Client::new(ClientOptions {
            url: format!("http://{}", self.addr.cluster.to_owned()),
            namespace: "default",
        })
        .unwrap();

        client.publish(vec![Event {
            user_id: "hubert@client".to_owned(),
            topic: "todo/1".to_owned(),
            name: "created".to_owned(),
            data: Some(pikav_client::Value {
                kind: Some(Kind::StructValue(Struct {
                    fields: HashMap::from([
                        (
                            "id".to_owned(),
                            Value {
                                kind: Some(Kind::Int32Value(1)),
                            },
                        ),
                        (
                            "text".to_owned(),
                            Value {
                                kind: Some(Kind::StringValue(
                                    "I don't want to work for somebody else".to_owned(),
                                )),
                            },
                        ),
                        (
                            "done".to_owned(),
                            Value {
                                kind: Some(Kind::BoolValue(true)),
                            },
                        ),
                    ]),
                })),
            }),
            metadata: None,
        }]);

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        Ok(())
    }
}
