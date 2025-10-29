mod config;
mod db;
mod models;
mod schema;

use crate::db::create_db_pool;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;


#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let _pool = create_db_pool().expect("Failed to create DB pool");

    let app = Router::new()
        .route("/", get(""));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Server listening on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERROR] Failed to start the server on {}: {}", addr, e);
            return;
        }
    };

    tracing::info!("Starting server on {}", addr);
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");
            tracing::info!("Shutting down gracefully...");
        })
        .await
    {
        eprintln!("[ERROR] Server error: {}", e);
    }
}
