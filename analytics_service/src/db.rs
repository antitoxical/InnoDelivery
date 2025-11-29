use crate::config::Config;
use clickhouse::Client as ClickHouseClient;
use mongodb::{options::ClientOptions, Client as MongoClient};
use std::time::Duration;

#[derive(Clone)]
pub struct DbConnection {
    pub mongo: MongoClient,
    pub clickhouse: ClickHouseClient,
}

pub async fn init_db(config: &Config) -> Result<DbConnection, Box<dyn std::error::Error>> {
    let mut client_options = ClientOptions::parse(&config.mongodb_uri).await?;
    client_options.app_name = Some("analytics_service".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(5));

    let mongo_client = MongoClient::with_options(client_options)?;

    mongo_client
        .database("admin")
        .run_command(mongodb::bson::doc! {"ping": 1})
        .await?;

    tracing::info!("Successfully connected to MongoDB");

    let clickhouse_client = ClickHouseClient::default()
        .with_url(&config.clickhouse_url)
        .with_user(&config.clickhouse_user)
        .with_password(&config.clickhouse_password)
        .with_database(&config.clickhouse_db);

    let _ = clickhouse_client
        .query("SELECT 1")
        .fetch_one::<u8>()
        .await?;
    tracing::info!("Successfully connected to ClickHouse");

    init_clickhouse_schema(&clickhouse_client).await?;

    Ok(DbConnection {
        mongo: mongo_client,
        clickhouse: clickhouse_client,
    })
}

async fn init_clickhouse_schema(
    client: &ClickHouseClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let ddl_orders = r#"
    CREATE TABLE IF NOT EXISTS orders_analytics (
        order_id String,
        user_id String,
        courier_id String,
        status String,
        total_price Float64,
        products Array(String),
        rating UInt8,
        finished_at DateTime DEFAULT now()
    ) ENGINE = MergeTree()
    ORDER BY (toYYYYMMDD(finished_at), courier_id);
    "#;
    client.query(ddl_orders).execute().await?;

    let ddl_users = r#"
    CREATE TABLE IF NOT EXISTS users_analytics (
        user_id String,
        role String,
        registered_at DateTime DEFAULT now()
    ) ENGINE = MergeTree()
    ORDER BY (toYYYYMMDD(registered_at), role);
    "#;
    client.query(ddl_users).execute().await?;

    tracing::info!("ClickHouse schema initialized");
    Ok(())
}
