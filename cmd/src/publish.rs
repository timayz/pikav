use std::time::Duration;

use config::{Config, ConfigError, Environment, File};
use pikav_client::{Client, ClientOptions, Event};
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
            namespace: None,
        })
        .unwrap();

        client.publish(vec![Event::new("hubert@client", "todo/1", "created", "i have work to do").unwrap()]);

        actix_rt::time::sleep(Duration::from_secs(1)).await;

        Ok(())
    }
}
