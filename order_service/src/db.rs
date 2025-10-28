use crate::config::CONFIG;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> Result<DbPool, String> {
    let manager = ConnectionManager::<PgConnection>::new(&CONFIG.database_url);
    Pool::builder().build(manager).map_err(|e| e.to_string())
}
