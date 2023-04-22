use std::str::FromStr;

use config::{Config, ConfigError, Environment, File};
use pikav_api::{client::Client, App, AppCors, AppJwks, AppOptions, Publisher};
use pikav_cluster::{Cluster, ClusterOptions};
use serde::Deserialize;
use tracing::Level;

#[derive(Debug, Deserialize)]
pub struct ServeAddr {
    pub api: String,
    pub cluster: String,
}

#[derive(Debug, Deserialize)]
pub struct Serve {
    pub addr: ServeAddr,
    pub cors: Option<AppCors>,
    pub jwks: Option<AppJwks>,
    pub nodes: Vec<String>,
    pub log: Option<String>,
}

impl Serve {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(File::with_name(path))
            .add_source(File::with_name(&format!("{path}.local")).required(false))
            .add_source(Environment::with_prefix(env!("CARGO_PKG_NAME")))
            .build()?
            .try_deserialize()
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let subscriber = tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(
                self.log
                    .as_ref()
                    .map(|log| Level::from_str(log).expect("failed to deserialize log"))
                    .unwrap_or(Level::ERROR),
            )
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");

        let nodes = match Client::from_vec(self.nodes.clone()) {
            Ok(nodes) => nodes,
            Err(e) => panic!("{e:?}"),
        };

        let publisher = Publisher::start();

        let cluster = Cluster::new(ClusterOptions {
            addr: self.addr.cluster.to_owned(),
            publisher: publisher.clone(),
            nodes: nodes.clone(),
        });

        let app = App::new(AppOptions {
            listen: self.addr.api.to_owned(),
            jwks: self.jwks.clone(),
            cors: self.cors.clone(),
            publisher,
            nodes,
        });

        actix_rt::spawn(async move { cluster.serve().await });

        app.run().await
    }
}
