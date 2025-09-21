use std::net::IpAddr;

use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use env_logger::Env;
use tracing::info;

use crate::library::api::proxy_websocket;
use crate::library::sql_lite::client::{new, wipe};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
mod library;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let debug_level = std::env::var("LOG_LEVEL").unwrap();

    let mut conn = new();

    let clean_tables = wipe(&mut conn);

    env_logger::init_from_env(Env::default().default_filter_or(debug_level));

    info!("Table Cleaned: {}", clean_tables);

    let default_address = String::from("127.0.0.1");
    let default_port = String::from("8000");
    let default_use_https = String::from("false");

    let bind_address: IpAddr = std::env::var("BIND_ADDRESS")
        .unwrap_or(default_address)
        .parse()
        .expect("Address not a valid address");
    let bind_port: u16 = std::env::var("BIND_PORT")
        .unwrap_or(default_port)
        .parse()
        .expect("Port not a valid");
    let use_https: bool = std::env::var("HTTPS_ENABLE")
        .unwrap_or(default_use_https)
        .parse()
        .expect("Value Not Boolean");

    if use_https {
        let cert = std::env::var("HTTPS_CERT_FILE").unwrap();
        let key = std::env::var("HTTPS_KEY_FILE").unwrap();
        let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();

        ssl_builder
            .set_private_key_file(key, SslFiletype::PEM)
            .unwrap();
        ssl_builder.set_certificate_chain_file(cert).unwrap();

        HttpServer::new(|| {
            App::new()
                .wrap(tracing_actix_web::TracingLogger::default())
                .wrap(Logger::default())
                .service(library::api::get_token::get_session_id)
                .service(proxy_websocket::ws_proxy)
        })
        .workers(4)
        .bind_openssl((bind_address, bind_port), ssl_builder)?
        .run()
        .await
    } else {
        HttpServer::new(|| {
            App::new()
                .wrap(tracing_actix_web::TracingLogger::default())
                .wrap(Logger::default())
                .service(library::api::get_token::get_session_id)
                .service(proxy_websocket::ws_proxy)
        })
        .workers(4)
        .bind((bind_address, bind_port))?
        .run()
        .await
    }
}
