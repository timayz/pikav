use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

cfg_if! {
if #[cfg(feature = "ssr")] {
    use sqlx::sqlite::SqlitePool;
    use actix_jwks::JwtPayload;
    use actix_web::{FromRequest, HttpRequest, rt::time::sleep};
    use rand::Rng;
    use pikav_client::Event;
    use std::time::Duration;
    use serde_json::json;

    #[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
    pub struct Todo {
        pub id: i64,
        pub user_id: String,
        pub text: String,
        pub done: bool,
    }

    #[derive(Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
    pub struct ReadTodo {
        pub id: i64,
        pub text: String,
        pub done: bool,
    }

    pub fn register_server_functions() {
        _ = GetTodos::register();
        _ = CreateTodo::register();
        _ = DeleteTodo::register();
    }
} else {

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Todo {
        pub id: i64,
        pub user_id: String,
        pub text: String,
        pub done: bool,
    }

    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ReadTodo {
        pub id: i64,
        pub text: String,
        pub done: bool,
    }
}
}

#[server(GetTodos, "/api")]
pub async fn get_todos(cx: Scope) -> Result<Vec<ReadTodo>, ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
    // let jwt_payload = JwtPayload::extract(&req)
    //     .await
    //     .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let jwt_payload = JwtPayload {
        subject: "yolo".into(),
        token: "val".into(),
        payload: "val".into(),
    };
    let pool = actix_web::web::Data::<SqlitePool>::extract(&req)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let todos = sqlx::query_as::<_, ReadTodo>(
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

    Ok(todos)
}

#[server(CreateTodo, "/api")]
async fn create_todo(cx: Scope, text: String) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
    let client = actix_web::web::Data::<pikav_client::Client>::extract(&req)
        .await
        .unwrap();
    // let jwt_payload = JwtPayload::extract(&req)
    //     .await
    //     .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let jwt_payload = JwtPayload {
        subject: "yolo".into(),
        token: "val".into(),
        payload: "val".into(),
    };
    let pool = actix_web::web::Data::<SqlitePool>::extract(&req)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let id = sqlx::query("INSERT INTO todos ( text, user_id ) VALUES ( ?1, ?2 )")
        .bind(text.to_owned())
        .bind(jwt_payload.subject.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![Event {
            user_id: jwt_payload.subject,
            topic: format!("todos/{id}"),
            name: "Created".to_owned(),
            data: Some(
                serde_json::to_value(ReadTodo {
                    done: false,
                    id,
                    text: text.to_owned(),
                })
                .unwrap()
                .into(),
            ),
            metadata: None,
        }]);
    });

    Ok(())
}

#[server(DeleteTodo, "/api")]
async fn delete_todo(cx: Scope, id: i64) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
    let client = actix_web::web::Data::<pikav_client::Client>::extract(&req)
        .await
        .unwrap();
    // let jwt_payload = JwtPayload::extract(&req)
    //     .await
    //     .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let jwt_payload = JwtPayload {
        subject: "yolo".into(),
        token: "val".into(),
        payload: "val".into(),
    };
    let pool = actix_web::web::Data::<SqlitePool>::extract(&req)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let rows_affected = sqlx::query("DELETE FROM todos WHERE id = ?1")
        .bind(id.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        return Err(ServerFnError::ServerError("Todo not found".into()));
    }

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![Event {
            user_id: jwt_payload.subject,
            topic: format!("todos/{id}"),
            name: "Deleted".to_owned(),
            data: Some(
                json!({
                    "id": id.to_owned()
                })
                .into(),
            ),
            metadata: None,
        }]);
    });

    Ok(())
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/example.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let create_todo = create_server_multi_action::<CreateTodo>(cx);
    let delete_todo = create_server_action::<DeleteTodo>(cx);
    let todos = create_resource(cx, || (), move |_| get_todos(cx));

    view! { cx,
        <h1>"Welcome to Pikav!"</h1>

        <MultiActionForm action=create_todo>
            <label>
                "Add a Todo"
                <input type="text" name="text" />
            </label>
            <input type="submit" value="Create" />
        </MultiActionForm>

        <Suspense fallback=move || view! { cx, <p>"Loading todos..."</p> }>
            <ul>
            {
                todos.with(cx, |todos| {
                    todos.clone().map(|todos| {
                        todos.into_iter().map(|todo|{
                            view! { cx,
                                <li>
                                    {&todo.text}
                                    <ActionForm action=delete_todo>
                                        <input type="hidden" name="id" value={todo.id} />
                                        <button type="submit">"X"</button>
                                    </ActionForm>
                                </li>
                            }
                        }).collect::<Vec<_>>()
                    })
                })
            }
            </ul>
        </Suspense>
    }
}