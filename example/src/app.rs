use cfg_if::cfg_if;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pikav_web::leptos::{pikav_context, use_subscribe};
use pikav_web::{Client, Headers};
use serde::{Deserialize, Serialize};

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use sqlx::sqlite::SqlitePool;
        use actix_web::{FromRequest, HttpRequest, rt::time::sleep};
        use rand::Rng;
        use pikav_client::Event;
        use std::time::Duration;
        use serde_json::json;

        pub fn register_server_functions() {
            _ = GetTodos::register();
            _ = CreateTodo::register();
            _ = DeleteTodo::register();
            _ = GetClientInfo::register();
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Todo {
    pub id: i64,
    pub user_id: String,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ReadTodo {
    pub id: i64,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientInfo {
    pub auth_token: String,
    pub endpoint: String,
}

#[server(GetTodos, "/api")]
pub async fn get_todos(cx: Scope, user_id: String) -> Result<Vec<ReadTodo>, ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
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
    .bind(user_id)
    .fetch_all(&mut conn)
    .await
    .unwrap();

    Ok(todos)
}

#[server(CreateTodo, "/api")]
async fn create_todo(cx: Scope, user_id: String, text: String) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
    let client = actix_web::web::Data::<pikav_client::Client>::extract(&req)
        .await
        .unwrap();
    let pool = actix_web::web::Data::<SqlitePool>::extract(&req)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let id = sqlx::query("INSERT INTO todos ( text, user_id ) VALUES ( ?1, ?2 )")
        .bind(text.to_owned())
        .bind(user_id.to_owned())
        .execute(&mut conn)
        .await
        .unwrap()
        .last_insert_rowid();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish(vec![Event {
            user_id,
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
async fn delete_todo(cx: Scope, user_id: String, id: i64) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>(cx).unwrap();
    let client = actix_web::web::Data::<pikav_client::Client>::extract(&req)
        .await
        .unwrap();
    let pool = actix_web::web::Data::<SqlitePool>::extract(&req)
        .await
        .unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let rows_affected = sqlx::query("DELETE FROM todos WHERE id = ?1 AND user_id = ?2")
        .bind(id.to_owned())
        .bind(user_id.to_owned())
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
            user_id,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResp {
    pub token_type: String,
    pub access_token: String,
}

#[server(GetClientInfo, "/api")]
async fn get_client_info(_cx: Scope, user_id: String) -> Result<ClientInfo, ServerFnError> {
    let resp: TokenResp = reqwest::Client::new()
        .post("http://127.0.0.1:6550/oauth/token")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "client_id": user_id }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    Ok(ClientInfo {
        auth_token: format!("{} {}", resp.token_type, resp.access_token),
        endpoint: format!(
            "http://127.0.0.1:{}",
            std::env::var("PIKAV_API_PORT").unwrap()
        ),
    })
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
        <Title text="Welcome to Pikav"/>

        // content for this welcome page
        <Router>
            
                <main>
                    <Routes>
                        <Route path="" view=|cx| view! { cx, <Configure><HomePage/></Configure>}/>
                    </Routes>
                </main>
            
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn Configure(cx: Scope, children: ChildrenFn) -> impl IntoView {
    // let query = use_query_map(cx);
    // let user_id = move || {
    //     query
    //         .with(|params| params.get("user").cloned())
    //         .unwrap_or("john".to_owned())
    // };

    let client = Client::new("http://127.0.0.1:6750")
        .namespace("example")
        .get_headers(move || {
            async move {
                let headers = Headers::new();
                headers.set("Authorization", "Bearer eyJraWQiOiJkZWZhdWx0LXJzYS1rZXktaWQiLCJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJhdWQiOiJkZWZhdWx0LWF1ZGllbmNlIiwic3ViIjoiam9obkBjbGllbnRzIiwiYXpwIjoiam9obiIsImlzcyI6ImRlZmF1bHQtaXNzdWVyIiwiZXhwIjoxNjg0ODgzODI4LCJndHkiOiJjbGllbnQtY3JlZGVudGlhbHMiLCJpYXQiOjE2ODQ4ODAyMjh9.IfFH_wJln-MBRQeL4kHNO6NwAuM3YL0fcDsNHn6nIUvIfbCW5Te3opo-Yn7Renyro0e8p_-WDF-Fdo0YSCzE5He15c4-cX7S_O7PlQUeI1MGheUNN-QTHQNWaSJmC-zjRqr1oLMy9P6GDr6-jNQiMxqcrda20o_O4eJVHGMQF-9tOKDvJjZojUKXLY3m6Io9C71PiTw0RpNUxCWZ-p5Hu_E6fWu_PwWoOqXm6rrbuJkw1unoRmGw0c5NrPwzDN6b3WrZGZY4oHrgEYybhsBFLAA1zapJNWN9TwYU6TsR8apv0AkrNPj8ZCIH3zeXcjeVJP9snzHho06kAr4mNrQQxg");
                Ok(headers)
            }
        })
        .run()
        .unwrap();

    pikav_context(cx, client);

    children(cx)

    // let client_info = create_resource(cx, user_id, move |user_id| get_client_info(cx, user_id));
    // let children = store_value(cx, children);

    // view! {cx,
    //     <Suspense fallback=|| ()>
    //         {move || client_info.with(cx, move |res| res.clone().map(|info| view! {cx, <ConfigurePikav info=info>{move || children.with_value(|children| children(cx))}</ConfigurePikav>}))}
    //     </Suspense>
    // }
}

// #[component]
// fn ConfigurePikav(cx: Scope, children: Children, info: ClientInfo) -> impl IntoView {
//     let client = Client::new(info.endpoint)
//         .namespace("example")
//         .get_headers(move || {
//             let token = info.auth_token.to_owned();
//             async move {
//                 let headers = Headers::new();
//                 headers.set("Authorization", token.as_str());
//                 Ok(headers)
//             }
//         })
//         .run()
//         .unwrap();

//     pikav_context(cx, client);

//     children(cx)
// }

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    let query = use_query_map(cx);
    let user_id = move || {
        let user = query
            .with(|params| params.get("user").cloned())
            .unwrap_or("john".to_owned());

        format!("{}@clients", user)
    };
    let create_todo = create_server_multi_action::<CreateTodo>(cx);
    let delete_todo = create_server_action::<DeleteTodo>(cx);
    let todos = create_resource(cx, user_id, move |user_id| get_todos(cx, user_id));

    use_subscribe(cx, "todos/*", move |e| async move {
        match e.name.as_str() {
            "Created" => {
                todos.update(move |res| {
                    let data = serde_json::from_value::<ReadTodo>(e.data).unwrap();
                    res.as_mut().unwrap().as_mut().unwrap().push(data);
                });
            }
            "Deleted" => {
                let id = e
                    .data
                    .as_object()
                    .unwrap()
                    .get("id")
                    .unwrap()
                    .as_i64()
                    .unwrap();

                todos.update(move |res| {
                    *res.as_mut().unwrap().as_mut().unwrap() = res
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .cloned()
                        .filter(|todo| todo.id != id)
                        .collect::<Vec<_>>();
                });
            }
            _ => {}
        }
    });

    view! { cx,
        <h1>"Welcome to Pikav!"</h1>

        <MultiActionForm action=create_todo>
            <label>
                "Add a Todo"
                <input type="text" name="text" />
                <input type="hidden" name="user_id" value={user_id()} />
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
                                        <input type="hidden" name="user_id" value={user_id()} />
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
