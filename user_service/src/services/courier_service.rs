use crate::dto::courier_dto::CourierProfileResponse;
use crate::models::courier::Courier;
use crate::models::courier::CourierStatus;
use crate::repository::courier_repository;
use crate::services::auth_service::AuthError;
use diesel::PgConnection;
use uuid::Uuid;

pub fn get_courier_profile(
    conn: &mut PgConnection,
    uid: Uuid,
) -> Result<CourierProfileResponse, AuthError> {
    let (user_model, courier_model) = courier_repository::find_courier_by_user_id(conn, uid)
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
    status_data: CourierStatus,
) -> Result<CourierStatus, AuthError> {
    let updated_courier = courier_repository::update_status_by_user_id(conn, uid, status_data)
        .map_err(|e| AuthError::DatabaseError(e.to_string()))?;
    Ok(updated_courier.status)
}

pub fn get_all_couriers(conn: &mut PgConnection) -> Result<Vec<CourierProfileResponse>, AuthError> {
    let results =
        courier_repository::get_all(conn).map_err(|e| AuthError::DatabaseError(e.to_string()))?;

    let profiles = results
        .into_iter()
        .map(|(user_model, courier_model)| CourierProfileResponse {
            id: user_model.id,
            name: user_model.name,
            phone_number: user_model.phone_number,
            email: user_model.email,
            status: courier_model.status,
            rating: courier_model.rating,
        })
        .collect();

    Ok(profiles)
}

pub fn set_courier_blocked_status(
    conn: &mut PgConnection,
    courier_user_id: Uuid,
    blocked: bool,
) -> Result<Courier, AuthError> {
    courier_repository::set_blocked_status(conn, courier_user_id, blocked).map_err(AuthError::from)
}

pub fn soft_delete_courier(
    conn: &mut PgConnection,
    courier_user_id: Uuid,
) -> Result<usize, AuthError> {
    courier_repository::soft_delete(conn, courier_user_id).map_err(AuthError::from)
}
