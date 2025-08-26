// src/db.rs

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn get_conn_from_pool(pool: &DbPool) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>, String> {
    pool.get().map_err(|e| e.to_string())
}
