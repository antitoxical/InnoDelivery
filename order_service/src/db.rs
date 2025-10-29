use crate::config::DATABASE_URL;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_db_pool() -> Result<DbPool, String> {
    let manager = ConnectionManager::<PgConnection>::new(&DATABASE_URL);
    Pool::builder().build(manager).map_err(|e| e.to_string())
}
