use crate::dto::user_dto::UpdateUser;
use crate::models::user::User as DbUser;
use crate::repository::user_repository;
use crate::services::auth_service::AuthError;
use diesel::PgConnection;
use uuid::Uuid;

pub fn get_user_profile(conn: &mut PgConnection, user_id: Uuid) -> Result<DbUser, AuthError> {
    user_repository::find_by_id(conn, user_id).map_err(AuthError::from)
}

pub fn update_user_profile(
    conn: &mut PgConnection,
    user_id: Uuid,
    update_data: UpdateUser,
) -> Result<DbUser, AuthError> {
    user_repository::update(conn, user_id, &update_data).map_err(AuthError::from)
}

pub fn soft_delete_user(conn: &mut PgConnection, user_id: Uuid) -> Result<usize, AuthError> {
    user_repository::soft_delete_user(conn, user_id).map_err(AuthError::from)
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<DbUser>, AuthError> {
    user_repository::get_all_users(conn).map_err(AuthError::from)
}

pub fn set_user_blocked_status(
    conn: &mut PgConnection,
    user_id: Uuid,
    blocked: bool,
) -> Result<DbUser, AuthError> {
    user_repository::set_blocked_status(conn, user_id, blocked).map_err(AuthError::from)
}
