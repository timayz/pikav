#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
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
            .service(Files::new("/", site_root))
        //.wrap(middleware::Compress::default())
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
