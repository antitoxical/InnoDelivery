// src/models/user.rs

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use crate::schema::users;

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub favorite_address: Option<String>,
    pub is_blocked: bool,
    pub is_deleted: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub password: String,
    pub role: String,
}
