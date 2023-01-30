use config::{Config, ConfigError, Environment, File};
use pikav_api::{client::Client, App, AppCors, AppJwks, AppOptions, Pikav};
use pikav_cluster::{Cluster, ClusterOptions};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServeCors {
    pub permissive: bool,
}

#[derive(Debug, Deserialize)]
pub struct ServeJwks {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ServeAddr {
    pub api: String,
    pub cluster: String,
}

#[derive(Debug, Deserialize)]
pub struct Serve {
    pub addr: ServeAddr,
    pub cors: Option<AppCors>,
    pub jwks: AppJwks,
    pub nodes: Vec<String>,
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
        let nodes = match Client::from_vec(self.nodes.clone()) {
            Ok(nodes) => nodes,
            Err(e) => panic!("{:?}", e),
        };

        let pikav = Pikav::new();

        let cluster = Cluster::new(ClusterOptions {
            addr: self.addr.cluster.to_owned(),
            pikav: pikav.clone(),
            nodes: nodes.clone(),
        });

        let app = App::new(AppOptions {
            listen: self.addr.api.to_owned(),
            jwks: self.jwks.clone(),
            cors: self.cors.clone(),
            pikav: pikav,
            nodes,
        });

        actix_rt::spawn(async move { cluster.serve().await });

        app.run().await
    }
}
