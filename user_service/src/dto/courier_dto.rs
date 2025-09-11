use crate::models::courier::CourierStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CourierProfileResponse {
    pub id: Uuid,
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub status: CourierStatus,
    pub rating: f32,
}
