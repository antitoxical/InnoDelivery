use crate::dto::courier_dto::{CourierProfileResponse, UpdateStatusDto};
use crate::models::courier::CourierStatus;
use crate::repository::courier_repository;
use crate::services::auth_service::AuthError;
use diesel::PgConnection;
use uuid::Uuid;

pub fn get_courier_profile(
    conn: &mut PgConnection,
    uid: Uuid,
) -> Result<CourierProfileResponse, AuthError> {
    let (user_model, courier_model) =
        courier_repository::find_courier_by_user_id(conn, uid)
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let response = CourierProfileResponse {
        id: user_model.id,
        name: user_model.name,
        phone_number: user_model.phone_number,
        email: user_model.email,
        status: courier_model.status,
        rating: courier_model.rating,
    };

    Ok(response)
}

pub fn update_courier_status(
    conn: &mut PgConnection,
    uid: Uuid,
    status_data: UpdateStatusDto,
) -> Result<CourierStatus, AuthError> {
    let updated_courier = courier_repository::update_status_by_user_id(
        conn,
        uid,
        status_data.status,
    )
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    Ok(updated_courier.status)
}

