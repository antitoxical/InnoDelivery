pub mod auth;
pub mod db;
pub mod dto;
pub mod handlers;
pub mod models;
pub mod services;
pub mod schema;

use actix_web::{web, App, HttpServer};
//mod handlers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Logger initialization
    tracing_subscriber::fmt::init();

    HttpServer::new(|| { App::new() })
        .bind("0.0.0.0:8080")?
        .run()
        .await
}