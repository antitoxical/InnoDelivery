use crate::schema::users;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct NewUser {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Name should contain from 2 to 100 characters"
    ))]
    pub name: String,
    pub phone_number: String,
    #[validate(email(message = "Incorrect e-mail format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must contain at least 6 characters"))]
    pub password: String,
    #[validate(length(
        min = 1,
        max = 50,
        message = "Role should be indicated and contain no more than 50 characters"
    ))]
    pub role: String,
}

#[derive(AsChangeset, Debug, Deserialize, Serialize, Validate)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    #[validate(length(
        min = 2,
        max = 100,
        message = "Name should contain from 2 to 100 characters"
    ))]
    pub name: Option<String>,
    pub phone_number: Option<String>,
    #[validate(email(message = "Incorrect e-mail format"))]
    pub email: Option<String>,
    pub favorite_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LoginUser {
    pub phone_number: String,
    #[validate(length(min = 6, message = "Password must contain at least 6 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
}
