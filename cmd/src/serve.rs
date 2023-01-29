use config::{Config, ConfigError, Environment, File};
use pikav_api::{
    client::{Client, ClusterOptions},
    App, AppOptions, Pikav,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Serve {
    pub listen: String,
    pub cors_permissive: Option<bool>,
    pub jwks_url: String,
    pub nodes: Vec<ClusterOptions>,
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
        let nodes = self
            .nodes
            .iter()
            .map(|node| Client::cluster(node.clone()))
            .collect();
        let pikav = Pikav::new();

        let app = App::new(AppOptions {
            listen: self.listen.to_owned(),
            jwks_url: self.jwks_url.to_owned(),
            cors_permissive: self.cors_permissive.unwrap_or(false),
            pikav,
            nodes,
        });

        app.run().await
    }
}
