mod admin;
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
use config::{config_admin, config_auth, config_courier, config_user};
use db::{DbPool, create_db_pool};
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
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

    if let Err(e) = admin::init_admin(&pool).await {
        tracing::error!("Failed to initialize admin user: {}", e);
        std::process::exit(1);
    }

    tracing::info!("Server created at http://127.0.0.1:8081");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(config_auth)
            .service(web::scope("/api").configure(config_user))
            .service(web::scope("/courier").configure(config_courier))
            .service(web::scope("/admin").configure(config_admin))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
