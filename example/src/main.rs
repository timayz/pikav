#[macro_use]
extern crate rbatis;

use actix_web::{
    delete, get, post, put, rt::time::sleep, web, App, HttpResponse, HttpServer, Responder,
};
use pikav_api::extractor::User;
use pikav_api::jwks::JwksClient;
use rand::Rng;
use rbatis::{crud::CRUD, rbatis::Rbatis};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

mod sqlite;

#[crud_table(table_name:todos)]
#[derive(Clone, Debug)]
pub struct Todo {
    pub id: Option<u64>,
    pub user_id: String,
    pub text: String,
    pub done: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadTodo {
    pub id: Option<u64>,
    pub text: String,
    pub done: bool,
}

impl From<Todo> for ReadTodo {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            text: todo.text,
            done: todo.done == 1,
        }
    }
}

#[get("/todos")]
async fn list(rb: web::Data<Arc<Rbatis>>, user: User) -> impl Responder {
    let v: Vec<ReadTodo> = rb
        .fetch_list_by_column::<Todo, _>("user_id", &[user.0])
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.into())
        .collect::<_>();

    HttpResponse::Ok().json(v)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateInput {
    text: String,
}

#[post("/todos")]
async fn create(
    rb: web::Data<Arc<Rbatis>>,
    client: web::Data<pikav_client::Client>,
    user: User,
    input: web::Json<CreateInput>,
) -> impl Responder {
    let todo = Todo {
        id: None,
        text: input.text.to_owned(),
        done: 0,
        user_id: user.0.to_owned(),
    };

    let res = rb.save(&todo, &[]).await.unwrap();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            user.0,
            format!("todos/{}", res.last_insert_id.unwrap()),
            "Created",
            todo,
        )
        .unwrap()]);
    });

    web::Json(json!({ "success": true }))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UpdateInput {
    text: String,
    done: bool,
}

#[put("/todos/{id}")]
async fn update(
    id: web::Path<String>,
    rb: web::Data<Arc<Rbatis>>,
    client: web::Data<pikav_client::Client>,
    user: User,
    input: web::Json<UpdateInput>,
) -> impl Responder {
    let result: Option<Todo> = rb.fetch_by_column("id", id.to_owned()).await.unwrap();

    let mut todo = match result {
        Some(todo) => todo,
        None => return HttpResponse::NotFound().json(json! ({ "success": false })),
    };

    if todo.user_id != user.0 {
        return HttpResponse::Forbidden().json(json! ({ "success": false }));
    }

    todo.text = input.text.to_owned();
    todo.done = if input.done { 1 } else { 0 };

    rb.update_by_column("id", &todo).await.ok();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            user.0,
            format!("todos/{}", id),
            "Updated",
            todo,
        )
        .unwrap()]);
    });

    HttpResponse::Ok().json(json! ({ "success": true }))
}

#[delete("/todos/{id}")]
async fn delete(
    id: web::Path<String>,
    rb: web::Data<Arc<Rbatis>>,
    client: web::Data<pikav_client::Client>,
    user: User,
) -> impl Responder {
    let result: Option<Todo> = rb.fetch_by_column("id", id.to_owned()).await.unwrap();

    let todo = match result {
        Some(todo) => todo,
        None => return HttpResponse::NotFound().json(json! ({ "success": false })),
    };

    if todo.user_id != user.0 {
        return HttpResponse::Forbidden().json(json! ({ "success": false }));
    }

    rb.remove_by_column::<Todo, _>("id", todo.id).await.ok();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            user.0,
            format!("todos/{}", id),
            "Deleted",
            json!({
                "id": id.to_owned()
            }),
        )
        .unwrap()]);
    });

    HttpResponse::Ok().json(json! ({ "success": true }))
}

#[actix_web::main] // or #[tokio::main]
async fn main() -> std::io::Result<()> {
    let rb = sqlite::init_sqlite_path("").await;
    let rb = Arc::new(rb);
    let jwks_client = JwksClient::new("http://127.0.0.1:4456/.well-known/jwks.json");
    let pikva_client = pikav_client::Client::new(pikav_client::ClientOptions {
        url: format!("http://127.0.0.1:{}", std::env::var("PIKAV_PORT").unwrap()),
        shared: None,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(rb.to_owned()))
            .app_data(web::Data::new(jwks_client.to_owned()))
            .app_data(web::Data::new(pikva_client.to_owned()))
            .service(list)
            .service(create)
            .service(update)
            .service(delete)
    })
    .bind(format!("0.0.0.0:{}", std::env::var("PORT").unwrap()))?
    .run()
    .await
}
