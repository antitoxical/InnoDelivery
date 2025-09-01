use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("DATABASE_URL must be set");

        Config {
            database_url,
            jwt_secret,
        }
    };
}
