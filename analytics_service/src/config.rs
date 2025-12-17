use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server_address: String,
    pub mongodb_uri: String,
    pub mongodb_db_name: String,
    pub clickhouse_url: String,
    pub clickhouse_user: String,
    pub clickhouse_password: String,
    pub clickhouse_db: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv().ok();
        Self {
            server_address: std::env::var("SERVER_ADDRESS")
                .unwrap_or_else(|_| "127.0.0.1:8082".to_string()),
            mongodb_uri: std::env::var("MONGODB_URI").expect("MONGODB_URI must be set"),
            mongodb_db_name: std::env::var("MONGODB_DB_NAME")
                .unwrap_or_else(|_| "analytics_db".to_string()),
            clickhouse_url: std::env::var("CLICKHOUSE_URL").expect("CLICKHOUSE_URL must be set"),
            clickhouse_user: std::env::var("CLICKHOUSE_USER")
                .unwrap_or_else(|_| "default".to_string()),
            clickhouse_password: std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_default(),
            clickhouse_db: std::env::var("CLICKHOUSE_DB").unwrap_or_else(|_| "default".to_string()),
        }
    }
}
