use serde::{Deserialize, Serialize};
use uuid::Uuid;
use clickhouse::Row;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub role: String,
    pub registered_at: DateTime<Utc>,
}

#[derive(Row, Serialize)]
pub struct UserAnalyticsRow {
    pub user_id: String,
    pub role: String,
    #[serde(with = "clickhouse_date_format")]
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OrderFinishedEvent {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub courier_id: Uuid,
    pub status: String,
    pub total_price: f64,
    pub products: Vec<String>,
    pub rating: u8,
    pub finished_at: DateTime<Utc>,
}

#[derive(Row, Serialize)]
pub struct OrderAnalyticsRow {
    pub order_id: String,
    pub user_id: String,
    pub courier_id: String,
    pub status: String,
    pub total_price: f64,
    pub products: Vec<String>,
    pub rating: u8,
    #[serde(with = "clickhouse_date_format")]
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawEvent<T> {
    pub event_type: String,
    pub payload: T,
    pub received_at: DateTime<Utc>,
}

mod clickhouse_date_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(date.timestamp() as u32)
    }
}