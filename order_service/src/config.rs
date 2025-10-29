use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref DATABASE_URL: String = {
        dotenvy::dotenv().ok();
        env::var("DATABASE_URL_ORDER").expect("DATABASE_URL_ORDER must be set")
    };
}
