mod config;
mod db;

mod graphql;
mod models;
mod repository;
mod schema;
mod services;

use crate::db::create_db_pool;
use crate::graphql::{graphql_handler, graphql_playground, AppSchema, MutationRoot, QueryRoot};
use axum::{routing::get, Extension, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let pool = create_db_pool().expect("Failed to create DB pool");

    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(30));
        loop {
            ticker.tick().await;
            tracing::info!("Running pending order processor...");
            match services::order_service::process_pending_orders(&pool_clone, 600).await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Processed {} pending orders.", count);
                    }
                }
                Err(e) => tracing::error!("Error processing pending orders: {:?}", e),
            }
        }
    });

    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pool)
        .finish();

    let app = Router::new()
        .route("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("GraphQL Playground listening on http://{}", addr);

    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ERROR] Failed to start the server on {}: {}", addr, e);
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
    {
        eprintln!("[ERROR] Server error: {}", e);
    }
}
