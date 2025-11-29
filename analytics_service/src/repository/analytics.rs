use crate::db::DbConnection;
use crate::models::event::{
    OrderAnalyticsRow, OrderFinishedEvent, RawEvent, UserAnalyticsRow, UserCreatedEvent,
};
use clickhouse::Client as ClickHouseClient;
use mongodb::{Client as MongoClient, bson::to_document};
use chrono::Utc;


pub async fn save_user_event(db: &DbConnection, event: UserCreatedEvent) -> Result<(), Box<dyn std::error::Error>> {
    let mongo_collection = db.mongo.database("analytics_db").collection("raw_events");
    let raw_event = RawEvent {
        event_type: "UserCreated".to_string(),
        payload: event.clone(),
        received_at: Utc::now(),
    };

    let doc = to_document(&raw_event)?;
    mongo_collection.insert_one(doc).await?;

    let row = UserAnalyticsRow {
        user_id: event.user_id.to_string(),
        role: event.role,
        registered_at: event.registered_at,
    };

    let mut insert = db.clickhouse.insert("users_analytics")?;
    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}


pub async fn save_order_event(db: &DbConnection, event: OrderFinishedEvent) -> Result<(), Box<dyn std::error::Error>> {
    let mongo_collection = db.mongo.database("analytics_db").collection("raw_events");
    let raw_event = RawEvent {
        event_type: "OrderFinished".to_string(),
        payload: event.clone(),
        received_at: Utc::now(),
    };
    let doc = to_document(&raw_event)?;
    mongo_collection.insert_one(doc).await?;

    let row = OrderAnalyticsRow {
        order_id: event.order_id.to_string(),
        user_id: event.user_id.to_string(),
        courier_id: event.courier_id.to_string(),
        status: event.status,
        total_price: event.total_price,
        products: event.products,
        rating: event.rating,
        finished_at: event.finished_at,
    };

    let mut insert = db.clickhouse.insert("orders_analytics")?;
    insert.write(&row).await?;
    insert.end().await?;

    Ok(())
}