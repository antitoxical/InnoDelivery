pub mod auth;
pub mod db;
pub mod dto;
pub mod handlers;
pub mod models;
pub mod services;
pub mod schema;
pub mod repository;
pub mod config;

use actix_web::{web, App, HttpServer};
use handlers::auth_handler::{register_user, login_user};
use handlers::user_handler::{get_profile, update_profile, delete_profile, get_users};
use db::{DbPool, create_db_pool};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let pool: DbPool = match create_db_pool() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("It was not possible to create a pool of the connections to the database: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Server created at http://127.0.0.1:8081");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register_user))
                    .route("/login", web::post().to(login_user))
            )
            .service(
                web::scope("/api")
                    .route("/profile", web::get().to(get_profile))
                    .route("/update", web::patch().to(update_profile))
                    .route("/delete", web::delete().to(delete_profile))
                    .route("/users", web::get().to(get_users))
            )
    })
        .bind(("127.0.0.1", 8081))?
        .run()
        .await
}
