mod config;
mod db;
mod handlers;
mod models;
mod repository;

use crate::config::Config;
use crate::db::init_db;
use crate::handlers::analytics::{track_order_completion, track_user_registration};
use axum::routing::post;
use axum::{routing::get, Router};
use mongodb::bson::doc;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub db: db::DbConnection,
    pub config: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env();
    tracing::info!("Starting Analytics Service");
    tracing::info!("Server Address: {}", config.server_address);
    tracing::info!("MongoDB URI: {}", config.mongodb_uri);

    tracing::info!("Initializing database connections...");
    let db_conn = init_db(&config).await.map_err(|e| {
        tracing::error!("Failed to connect to databases: {}", e);
        e
    })?;
    tracing::info!("Database pool created.");

    tracing::info!("Pinging MongoDB from main...");
    if let Err(e) = db_conn
        .mongo
        .database("admin")
        .run_command(doc! {"ping": 1})
        .await
    {
        tracing::error!("MongoDB Ping FAILED: {}", e);
        return Err(e.into());
    }
    tracing::info!("MongoDB Ping successful! Connection is active.");

    let state = AppState {
        db: db_conn,
        config: config.clone(),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/internal/analytics/user", post(track_user_registration))
        .route("/internal/analytics/order", post(track_order_completion))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = config.server_address.parse()?;
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Analytics Service is operational"
}
