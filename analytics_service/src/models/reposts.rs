use clickhouse::Row;
use serde::Serialize;

#[derive(Row, Serialize)]
pub struct OrdersByDate {
    pub date: String,
    pub count: u64,
}

#[derive(Row, Serialize)]
pub struct CourierRating {
    pub courier_id: String,
    pub avg_rating: f64,
}

#[derive(Row, Serialize)]
pub struct TopProduct {
    pub product: String,
    pub count: u64,
}
