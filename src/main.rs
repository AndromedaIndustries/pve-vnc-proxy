use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use tracing::info;

use crate::library::api::proxy_websocket;
use crate::library::sql_lite::client::{new, wipe};
mod library;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let debug_level = std::env::var("LOG_LEVEL").unwrap();

    let mut conn = new();

    let clean_tables = wipe(&mut conn);

    env_logger::init_from_env(Env::default().default_filter_or(debug_level));

    info!("Table Cleaned: {}", clean_tables);

    HttpServer::new(|| {
        App::new()
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(Logger::default())
            .service(library::api::get_token::get_session_id)
            .service(proxy_websocket::ws_proxy)
    })
    .workers(4)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
