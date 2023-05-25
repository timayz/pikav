use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use serde::{Serialize, Deserialize};

        #[derive(Debug, Serialize, Deserialize)]
        pub struct TokenResp {
            pub token_type: String,
            pub access_token: String,
        }

        #[derive(Debug, Deserialize)]
        pub struct Params {
            user: Option<String>,
        }
    }
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use actix_web::{cookie::Cookie, dev::Service};
    use example::app::*;
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use sqlx::sqlite::SqlitePool;

    let pikv_client = pikav_client::Client::new(pikav_client::ClientOptions {
        url: format!("http://127.0.0.1:{}", std::env::var("PIKAV_PORT").unwrap()),
        namespace: "example",
    })
    .unwrap();

    let pool = SqlitePool::connect("sqlite://target/todos.db?mode=rwc")
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();

    example::app::register_server_functions();

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> });

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;

        App::new()
            .app_data(web::Data::new(pool.to_owned()))
            .app_data(web::Data::new(pikv_client.to_owned()))
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(
                leptos_options.to_owned(),
                routes.to_owned(),
                |cx| view! { cx, <App/> },
            )
            .wrap_fn(|req, srv| {
                let accept_html = req
                    .headers()
                    .get("Accept")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.contains("text/html"))
                    .unwrap_or(false);

                let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
                let user = params.user.to_owned().unwrap_or("john".to_owned());

                let fut = srv.call(req);

                Box::pin(async move {
                    let mut res = fut.await?;

                    if !accept_html {
                        return Ok(res);
                    }

                    let response = res.response_mut();

                    let token_resp: TokenResp = reqwest::Client::new()
                        .post("http://127.0.0.1:6550/oauth/token")
                        .header("Accept", "application/json")
                        .header("Content-Type", "application/json")
                        .json(&serde_json::json!({ "client_id": user }))
                        .send()
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();

                    let auth_token =
                        format!("{} {}", token_resp.token_type, token_resp.access_token);

                    let _ = response.add_cookie(&Cookie::build("auth_token", auth_token).finish());

                    let endpoint = format!(
                        "http://127.0.0.1:{}",
                        std::env::var("PIKAV_API_PORT").unwrap()
                    );

                    let _ = response.add_cookie(&Cookie::build("endpoint", endpoint).finish());

                    Ok(res)
                })
            })
            .service(Files::new("/", site_root))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
