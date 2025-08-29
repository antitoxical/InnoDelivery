use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use crate::config;
use crate::config::DATABASE_URL;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> Result<DbPool, String> {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL.as_ref());
    Pool::builder().build(manager).map_err(|e| e.to_string())
}


pub fn get_conn_from_pool(pool: &DbPool) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>, String> {
    pool.get().map_err(|e| e.to_string())
}
