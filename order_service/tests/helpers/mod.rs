use axum::{routing::post, Extension, Router};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use httpmock::MockServer;
use order_service::db::DbPool;
use order_service::graphql::{graphql_handler, AppSchema, MutationRoot, QueryRoot};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_order");

pub async fn setup_test_app(database_url: &str) -> (Router, DbPool, MockServer) {
    let _ = dotenvy::dotenv();

    let server = MockServer::start_async().await;

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = Pool::builder()
        .build(manager)
        .expect("Failed to create test DB pool");

    {
        let mut conn = pool.get().expect("Failed to get DB conn for setup");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
        diesel::sql_query("TRUNCATE TABLE order_products, orders, products CASCADE")
            .execute(&mut conn)
            .expect("Failed to truncate tables");
    }

    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pool.clone())
        .data(server.base_url())
        .finish();

    let app = Router::new()
        .route("/", post(graphql_handler))
        .layer(Extension(schema));

    (app, pool, server)
}
