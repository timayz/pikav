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

        #[derive(Debug, Serialize, Deserialize)]
        pub struct TokenResp {
            pub token_type: String,
            pub access_token: String,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ClientInfo {
    pub auth_token: String,
    pub endpoint: String,
}

#[server(GetClientInfo, "/api")]
pub async fn get_client_info(user_id: String) -> Result<ClientInfo, ServerFnError> {
    let token_resp: TokenResp = reqwest::Client::new()
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

    let auth_token = format!("{} {}", token_resp.token_type, token_resp.access_token);

    let endpoint = format!(
        "http://127.0.0.1:{}",
        std::env::var("PIKAV_API_PORT").unwrap()
    );

    Ok(ClientInfo {
        auth_token,
        endpoint,
    })
}

#[server(GetTodos, "/api")]
pub async fn get_todos(user_id: String) -> Result<Vec<ReadTodo>, ServerFnError> {
    let req = use_context::<HttpRequest>().unwrap();
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
    .fetch_all(&mut *conn)
    .await
    .unwrap();

    Ok(todos)
}

#[server(CreateTodo, "/api")]
async fn create_todo(user_id: String, text: String) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>().unwrap();
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
        .execute(&mut *conn)
        .await
        .unwrap()
        .last_insert_rowid();

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish_events(vec![Event {
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
async fn delete_todo(user_id: String, id: i64) -> Result<(), ServerFnError> {
    let req = use_context::<HttpRequest>().unwrap();
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
        .execute(&mut *conn)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        return Err(ServerFnError::ServerError("Todo not found".into()));
    }

    actix_web::rt::spawn(async move {
        let mut rng = rand::thread_rng();

        sleep(Duration::from_secs(rng.gen_range(0..3))).await;

        client.publish_events(vec![Event {
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

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/example.css"/>

        // sets the document title
        <Title text="Welcome to Pikav"/>

        // content for this welcome page
        <Router>
            <AppConfig>
                <Pikav>
                  <main>
                      <Routes>
                          <Route path="" view=|| view! { <HomePage/>}/>
                      </Routes>
                  </main>
                </Pikav>
            </AppConfig>
        </Router>
    }
}

#[component]
fn AppConfig(children: ChildrenFn) -> impl IntoView {
    let query = use_query_map();
    let user_id = move || {
        query
            .with(|params| params.get("user").cloned())
            .unwrap_or("john".to_owned())
    };

    let app_config = create_resource(user_id, get_client_info);
    let children = store_value(children);

    view! {
      <Suspense fallback=|| ()>
      {move ||
          app_config.and_then(|config| {
                provide_context(config.clone());

                children.with_value(|children| children())
            })
      }
      </Suspense>
    }
}

#[component]
fn Pikav(children: Children) -> impl IntoView {
    let info = use_context::<ClientInfo>().unwrap_or_default();
    let client = Client::new(info.endpoint)
        .namespace("example")
        .get_headers(move || {
            let auth_token = info.auth_token.to_owned();
            async move {
                let headers = Headers::new();
                headers.set("Authorization", &auth_token);
                Ok(headers)
            }
        })
        .run()
        .unwrap();

    pikav_context(client);

    children()
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let query = use_query_map();
    let user_id = move || {
        let user = query
            .with(|params| params.get("user").cloned())
            .unwrap_or("john".to_owned());

        format!("{}@clients", user)
    };
    let create_todo = create_server_multi_action::<CreateTodo>();
    let delete_todo = create_server_action::<DeleteTodo>();
    let todos = create_resource(user_id, get_todos);

    use_subscribe("todos/*", move |e| async move {
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
                        .iter().filter(|&todo| todo.id != id).cloned()
                        .collect::<Vec<_>>();
                });
            }
            _ => {}
        }
    });

    view! {
        <h1>"Welcome to Pikav!"</h1>

        <MultiActionForm action=create_todo>
            <label>
                "Add a Todo"
                <input type="text" name="text" />
                <input type="hidden" name="user_id" value={user_id()} />
            </label>
            <input type="submit" value="Create" />
        </MultiActionForm>

        <Suspense fallback=move || view! { <p>"Loading todos..."</p> }>
            <ul>
            {move ||
                todos.and_then(|data| {
                    data.iter().map(|todo|{
                        view! {
                            <li>
                                {&todo.text}
                                <ActionForm action=delete_todo clone:todo>
                                    <input type="hidden" name="user_id" value={user_id()} />
                                    <input type="hidden" name="id" value={todo.id} />
                                    <button type="submit">"X"</button>
                                </ActionForm>
                            </li>
                        }
                    }).collect_view()
                })
            }
            </ul>
        </Suspense>
    }
}
