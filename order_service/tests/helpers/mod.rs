use axum::{routing::post, Extension, Router};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel::RunQueryDsl;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use httpmock::MockServer;
use order_service::config;
use order_service::db::DbPool;
use order_service::graphql::{graphql_handler, AppSchema, MutationRoot, QueryRoot};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_order");

fn run_migrations(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
}

fn clear_db(conn: &mut PgConnection) {
    diesel::sql_query("TRUNCATE TABLE order_products, orders, products CASCADE")
        .execute(conn)
        .expect("Failed to truncate tables");
}

pub fn create_test_pool() -> DbPool {
    dotenvy::dotenv().ok();
    let database_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create test DB pool")
}

pub async fn setup_test_app() -> (Router, DbPool, MockServer) {
    dotenvy::dotenv().ok();
    let pool = create_test_pool();

    {
        let mut conn = pool.get().expect("Failed to get DB conn for setup");
        run_migrations(&mut conn);
        clear_db(&mut conn);
    }

    let server = MockServer::start();
    env::set_var("USER_SERVICE_URL", &*config::user_service_url);

    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pool.clone())
        .finish();

    let app = Router::new()
        .route("/", post(graphql_handler))
        .layer(Extension(schema));

    (app, pool, server)
}
