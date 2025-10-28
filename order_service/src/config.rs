use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub database_url: String,
    //pub pending_order_check_interval_ms: u64,
    //pub pending_order_timeout_seconds: i64,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL_ORDER").expect("DATABASE_URL_ORDER must be set");

        /*let pending_order_check_interval_ms = env::var("PENDING_ORDER_CHECK_INTERVAL_MS")
            .unwrap_or_else(|_| "60000".to_string())
            .parse::<u64>()
            .expect("PENDING_ORDER_CHECK_INTERVAL_MS must be a number");

        let pending_order_timeout_seconds = env::var("PENDING_ORDER_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<i64>()
            .expect("PENDING_ORDER_TIMEOUT_SECONDS must be a number");*/

        Config {
            database_url,
            /*pending_order_check_interval_ms,
            pending_order_timeout_seconds,*/
        }
    };
}
