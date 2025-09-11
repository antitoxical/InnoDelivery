use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUser {
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(AsChangeset, Debug, Deserialize, Serialize)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub favorite_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUser {
    pub phone_number: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
}
