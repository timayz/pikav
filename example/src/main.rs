use actix_jwks::{JwksClient, JwtPayload};
use actix_web::{
    delete, get, post, put, rt::time::sleep, web, App, HttpResponse, HttpServer, Responder,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::sqlite::SqlitePool;
use std::time::Duration;

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub user_id: String,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct ReadTodo {
    pub id: i64,
    pub text: String,
    pub done: bool,
}

#[get("/todos")]
async fn list(pool: web::Data<SqlitePool>, jwt_payload: JwtPayload) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();

    let v = sqlx::query_as::<_, ReadTodo>(
        r#"
SELECT id, text, done
FROM todos
WHERE user_id = ?1
        "#,
    )
    .bind(jwt_payload.subject)
    .fetch_all(&mut conn)
    .await
    .unwrap();

    HttpResponse::Ok().json(v)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CreateInput {
    text: String,
}

#[post("/todos")]
async fn create(
    pool: web::Data<SqlitePool>,
    client: web::Data<pikav_client::Client>,
    jwt_payload: JwtPayload,
    input: web::Json<CreateInput>,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();

    let id = sqlx::query("INSERT INTO todos ( text, user_id ) VALUES ( ?1, ?2 )")
        .bind(input.text.to_owned())
        .bind(jwt_payload.subject.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            jwt_payload.subject,
            format!("todos/{}", id),
            "Created",
            ReadTodo {
                done: false,
                id,
                text: input.text.to_owned(),
            },
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
    id: web::Path<i64>,
    pool: web::Data<SqlitePool>,
    client: web::Data<pikav_client::Client>,
    jwt_payload: JwtPayload,
    input: web::Json<UpdateInput>,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();

    let rows_affected = sqlx::query("UPDATE todos SET done = ?2, text = ?3 WHERE id = ?1")
        .bind(id.to_owned())
        .bind(input.done)
        .bind(input.text.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        return HttpResponse::NotFound().json(json! ({ "success": false }));
    }

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            jwt_payload.subject,
            format!("todos/{}", id),
            "Updated",
            ReadTodo {
                id: id.to_owned(),
                text: input.text.to_owned(),
                done: input.done,
            },
        )
        .unwrap()]);
    });

    HttpResponse::Ok().json(json! ({ "success": true }))
}

#[delete("/todos/{id}")]
async fn delete(
    id: web::Path<i64>,
    pool: web::Data<SqlitePool>,
    client: web::Data<pikav_client::Client>,
    jwt_payload: JwtPayload,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();

    let rows_affected = sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        return HttpResponse::NotFound().json(json! ({ "success": false }));
    }

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![pikav_client::Event::new(
            jwt_payload.subject,
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
    let jwks_client = JwksClient::new("http://127.0.0.1:4456/.well-known/jwks.json");
    let pikva_client = pikav_client::Client::new(pikav_client::ClientOptions {
        url: format!("http://127.0.0.1:{}", std::env::var("PIKAV_PORT").unwrap()),
        namespace: None,
    });

    let pool = SqlitePool::connect("sqlite://target/todos.db")
        .await
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.to_owned()))
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
