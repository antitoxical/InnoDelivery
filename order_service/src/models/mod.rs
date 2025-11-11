use crate::schema::{order_products, orders, products};
use async_graphql::{Enum, SimpleObject};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::OrderStatus"]
pub enum OrderStatus {
    Draft,
    PendingCarrier,
    InProgress,
    Finished,
    Cancelled,
}

#[derive(SimpleObject, Queryable, Selectable, Identifiable, Serialize, Debug)]
#[diesel(table_name = orders)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub courier_id: Option<Uuid>,
    pub delivery_address: String,
    pub status: OrderStatus,
    pub rating: Option<f32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = orders)]
pub struct NewOrder {
    pub user_id: Uuid,
    pub courier_id: Option<Uuid>,
    pub delivery_address: String,
    pub status: OrderStatus,
}

#[derive(Queryable, Selectable, Identifiable, Serialize, Debug)]
#[diesel(table_name = products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Product {
    pub id: Uuid,
    pub product_type: String,
    pub product_name: String,
    pub restaurant: String,
    pub price: f32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Associations, Serialize, Debug)]
#[diesel(belongs_to(Order))]
#[diesel(belongs_to(Product))]
#[diesel(table_name = order_products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OrderProduct {
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = order_products)]
pub struct NewOrderProduct {
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
}
