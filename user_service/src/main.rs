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
use db::{DbPool, create_db_pool};
use handlers::auth_handler::{login_user, register_user};
use handlers::courier_handler::{get_courier_profile, update_courier_status};
use handlers::user_handler::{delete_profile, get_profile, get_users, update_profile};

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
            .service(
                web::scope("/auth")
                    .route("/register", web::post().to(register_user))
                    .route("/login", web::post().to(login_user)),
            )
            .service(
                web::scope("/api")
                    .route("/profile", web::get().to(get_profile))
                    .route("/update", web::patch().to(update_profile))
                    .route("/delete", web::delete().to(delete_profile))
                    .route("/users", web::get().to(get_users)),
            )
            .service(
                web::scope("/courier")
                    .route("/profile", web::get().to(get_courier_profile))
                    .route("/status", web::patch().to(update_courier_status)),
            )
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
