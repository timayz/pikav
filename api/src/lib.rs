mod error;

pub mod extractor;

use std::collections::HashSet;

use actix_cors::Cors;
use actix_jwks::{JwksClient, JwtPayload};
use actix_web::{
    error::ErrorInternalServerError,
    get,
    middleware::Condition,
    post, put,
    web::{self, Bytes, Data},
    App as ActixApp, Error as ActixError, HttpRequest, HttpResponse, HttpServer,
};
use error::ApiError;
use extractor::{Client as ReqClient, PikavInfo};
use futures_core::Stream;
use futures_util::future::join_all;
use pikav::{
    topic::{TopicFilter, TopicName},
    Event, PubEvent,
};
use serde::Deserialize;
use serde_json::json;
use tracing::error;

pub use pikav::{Pikav, Receiver, SubscribeOptions};
pub use pikav_client as client;

// struct Forward(Data<Vec<client::Client>>);

// lazy_static::lazy_static! {
//     static ref HOP_HEADERS: HashSet<&'static str> = {
//         let mut m = HashSet::new();
//         m.insert("connection");
//         m.insert("keep-alive");
//         m.insert("proxy-authenticate");
//         m.insert("proxy-authorization");
//         m.insert("te");
//         m.insert("trailers");
//         m.insert("transfer-encoding");
//         m.insert("upgrade");
//         m
//     };
// }

// impl Forward {
//     pub async fn execute<'a>(&self, req: &'a HttpRequest) {
//         let shared_nodes = self.0.iter().filter(|n| n.is_shared());
//         let futures = shared_nodes.map(|n| {
//             let mut client_req = n.put(req.uri());
//             let headers = client_req.headers_mut();

//             for (key, val) in req.headers() {
//                 if !HOP_HEADERS.contains(key.as_str()) && !headers.contains_key(key) {
//                     headers.insert(key.to_owned(), val.to_owned());
//                 }
//             }

//             client_req.send()
//         });

//         for res in join_all(futures).await {
//             if let Err(e) = res {
//                 error!("Forward {} :: {e}", req.uri());
//             }
//         }
//     }
// }

#[put(r"/subscribe/{filter:.*}")]
async fn subscribe(
    params: web::Path<(String,)>,
    pikav: Data<Pikav<Bytes>>,
    client: ReqClient,
    nodes: Data<Vec<client::Client>>,
    jwt: JwtPayload,
    info: PikavInfo,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let params = params.into_inner();
    let filter = TopicFilter::new(params.0.to_owned())?;

    pikav
        .subscribe(SubscribeOptions {
            filter,
            user_id: jwt.subject,
            client_id: client.0.to_owned(),
        })
        .ok();

    // if !info.is_cluster() {
    //     Forward(nodes).execute(&req).await;
    // }

    Ok(HttpResponse::Ok().json(json! ({ "success": true })))
}

#[put(r"/unsubscribe/{filter:.*}")]
async fn unsubscribe(
    params: web::Path<(String,)>,
    pikav: Data<Pikav<Bytes>>,
    client: ReqClient,
    jwt: JwtPayload,
    nodes: Data<Vec<client::Client>>,
    info: PikavInfo,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let params = params.into_inner();
    let filter = TopicFilter::new(params.0.to_owned())?;

    pikav
        .unsubscribe(SubscribeOptions {
            filter,
            user_id: jwt.subject,
            client_id: client.0,
        })
        .ok();

    // if !info.is_cluster() {
    //     Forward(nodes).execute(&req).await;
    // }

    Ok(HttpResponse::Ok().json(json! ({ "success": true })))
}

// #[post("/publish/{namespace}")]
// async fn publish(
//     namespace: web::Path<String>,
//     pikav: Data<Pikav<Bytes>>,
//     nodes: Data<Vec<client::Client>>,
//     input: web::Json<Vec<client::Event>>,
//     info: PikavInfo,
// ) -> Result<HttpResponse, ApiError> {
//     let mut pub_events = Vec::new();

//     for e in input.iter() {
//         let topic =
//             TopicName::new(e.topic.to_owned()).map_err(|e| ApiError::BadRequest(e.to_string()))?;

//         pub_events.push(PubEvent {
//             event: Event {
//                 topic,
//                 name: e.name.to_owned(),
//                 data: e.data.clone(),
//                 metadata: e.metadata.clone(),
//             },
//             user_id: format!("{}/{}", namespace, e.user_id),
//         });
//     }

//     pikav.publish(pub_events.iter().collect::<_>());

//     if !info.is_cluster() {
//         for node in nodes.iter() {
//             node.publish(input.clone());
//         }
//     }

//     Ok(HttpResponse::Ok().json(json! ({ "success": true })))
// }

#[get("/events")]
async fn events(pikav: Data<Pikav<Bytes>>) -> Result<HttpResponse, ApiError> {
    let rx = match pikav.new_client() {
        Some(rx) => rx,
        None => {
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
    pub jwks: AppJwks,
    pub cors: Option<AppCors>,
    pub pikav: Pikav<Bytes>,
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
        let pikav = self.options.pikav.clone();

        let cors_permissive = self
            .options
            .cors
            .as_ref()
            .map(|cors| cors.permissive)
            .unwrap_or_default();

        let jwks_client = JwksClient::new(self.options.jwks.url.to_owned());
        let nodes = self.options.nodes.clone();

        HttpServer::new(move || {
            ActixApp::new()
                .app_data(Data::new(pikav.clone()))
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
