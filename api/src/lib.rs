mod error;

pub mod extractor;

use std::io::Error;

use actix_cors::Cors;
use actix_web::{
    error::ErrorInternalServerError,
    get,
    middleware::Condition,
    put,
    web::{self, Bytes, Data},
    App as ActixApp, Error as ActixError, HttpResponse, HttpServer,
};
use client::{SubscribeRequest, UnsubscribeRequest};
use error::ApiError;
use extractor::Client as ReqClient;
use futures_core::Stream;
use serde::Deserialize;
use serde_json::json;
use twa_jwks::{actix_web::JwtPayload, JwksClient};

pub use pikav::publisher::{Publisher, Receiver};
pub use pikav_client as client;

#[derive(Deserialize)]
struct JwtClaims {
    pub sub: String,
}

#[put(r"/subscribe/{filter:.*}")]
async fn subscribe(
    params: web::Path<(String,)>,
    publisher: Data<Publisher<Bytes>>,
    client: ReqClient,
    nodes: Data<Vec<client::Client>>,
    JwtPayload(payload): JwtPayload<JwtClaims>,
) -> Result<HttpResponse, ApiError> {
    let params = params.into_inner();

    publisher
        .subscribe(params.0.to_owned(), &payload.sub, &client.0)
        .await
        .ok();

    for node in nodes.iter().filter(|n| n.same_region) {
        node.subscribe(SubscribeRequest {
            filter: params.0.to_owned(),
            client_id: client.0.to_owned(),
            user_id: payload.sub.to_owned(),
        })
        .await?;
    }

    Ok(HttpResponse::Ok().json(json! ({ "success": true })))
}

#[put(r"/unsubscribe/{filter:.*}")]
async fn unsubscribe(
    params: web::Path<(String,)>,
    publisher: Data<Publisher<Bytes>>,
    client: ReqClient,
    JwtPayload(payload): JwtPayload<JwtClaims>,
    nodes: Data<Vec<client::Client>>,
) -> Result<HttpResponse, ApiError> {
    let params = params.into_inner();

    publisher
        .unsubscribe(params.0.to_owned(), &payload.sub, &client.0)
        .await
        .ok();

    for node in nodes.iter().filter(|n| n.same_region) {
        node.unsubscribe(UnsubscribeRequest {
            filter: params.0.to_owned(),
            client_id: client.0.to_owned(),
            user_id: payload.sub.to_owned(),
        })
        .await?;
    }

    Ok(HttpResponse::Ok().json(json! ({ "success": true })))
}

#[get("/events")]
async fn events(publisher: Data<Publisher<Bytes>>) -> Result<HttpResponse, ApiError> {
    let rx = match publisher.create_client().await {
        Some(rx) => rx,
        _ => {
            return ApiError::InternalServerError("Failed to create client".to_owned())
                .into_response()
        }
    };

    Ok(HttpResponse::Ok()
        .append_header(("Content-Type", "text/event-stream"))
        .streaming(Client(rx)))
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppCors {
    pub permissive: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppJwks {
    pub url: String,
}

pub struct AppOptions {
    pub listen: String,
    pub jwks: Option<AppJwks>,
    pub cors: Option<AppCors>,
    pub publisher: Publisher<Bytes>,
    pub nodes: Vec<client::Client>,
}

pub struct App {
    pub options: AppOptions,
}

impl App {
    pub fn new(options: AppOptions) -> Self {
        Self { options }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let publisher = self.options.publisher.clone();

        let cors_permissive = self
            .options
            .cors
            .as_ref()
            .map(|cors| cors.permissive)
            .unwrap_or_default();

        let jwks_client =
            JwksClient::build(self.options.jwks.as_ref().map(|jwks| jwks.url.to_owned()))
                .await
                .map_err(|e| Error::new(std::io::ErrorKind::Other, e))?;

        let nodes = self.options.nodes.clone();

        println!(
            "Pikav api server listening on {}",
            self.options.listen.to_owned()
        );

        HttpServer::new(move || {
            ActixApp::new()
                .app_data(Data::new(publisher.clone()))
                .app_data(Data::new(jwks_client.clone()))
                .app_data(Data::new(nodes.clone()))
                .wrap(Condition::new(cors_permissive, Cors::permissive()))
                .service(subscribe)
                .service(unsubscribe)
                .service(events)
        })
        .bind(self.options.listen.to_owned())?
        .run()
        .await
    }
}

pub struct Client(Receiver<Bytes>);

impl Stream for Client {
    type Item = Result<Bytes, ActixError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.0
            .poll_recv(cx)
            .map(|res| Some(res.ok_or_else(|| ErrorInternalServerError(""))))
    }
}
