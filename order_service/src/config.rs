use lazy_static::lazy_static;
use std::env;

lazy_static! {
    pub static ref DATABASE_URL: String =
        env::var("DATABASE_URL_ORDER").expect("DATABASE_URL_ORDER must be set");
    pub static ref user_service_url: String =
        env::var("USER_SERVICE_URL").unwrap_or("http://127.0.0.1:8081/".to_string());
    pub static ref DATABASE_URL_ORDER_TEST: String =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");
}
