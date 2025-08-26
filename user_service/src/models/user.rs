use crate::schema::{users, couriers};
use diesel::prelude::*;
use chrono::NaiveDateTime;
use serde::{Serialize, Deserialize};

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct Users {
    pub id: String,
    pub first_name: String,
    pub address: String,
    pub phone_number: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub is_blocked: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime
}
#[derive(Insertable, Debug)]
#[diesel(belongs_to(Users))]
#[diesel(table_name = courier)]
pub struct Couriers {
    pub id: String,
    pub is_free: bool,
    pub rating: f64,
}
