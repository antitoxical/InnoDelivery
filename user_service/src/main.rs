mod auth;
mod config;
mod db;
mod dto;
mod handlers;
mod models;
mod repository;
mod schema;
mod services;

use actix_web::{App, HttpServer, web};
use config::{config_auth, config_courier, config_user};
use db::{DbPool, create_db_pool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let pool: DbPool = match create_db_pool() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(
                "It was not possible to create a pool of the connections to the database: {}",
                e
            );
            std::process::exit(1);
        }
    };

    tracing::info!("Server created at http://127.0.0.1:8081");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(config_auth)
            .service(web::scope("/api").configure(config_user))
            .service(web::scope("/courier").configure(config_courier))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
