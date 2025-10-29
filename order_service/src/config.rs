use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub database_url: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL_ORDER").expect("DATABASE_URL_ORDER must be set");
        Config { database_url }
    };
}
