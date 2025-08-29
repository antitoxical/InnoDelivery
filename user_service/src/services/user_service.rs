use crate::dto::user_dto::UpdateUser;
use crate::models::user::User as DbUser;
use crate::repository::repository;
use crate::services::auth_service::AuthError;
use diesel::PgConnection;
use uuid::Uuid;

pub fn get_user_profile(conn: &mut PgConnection, user_id: Uuid) -> Result<DbUser, AuthError> {
    repository::find_by_id(conn, user_id)
}

pub fn update_user_profile(
    conn: &mut PgConnection,
    user_id: Uuid,
    update_data: UpdateUser,
) -> Result<DbUser, AuthError> {
    repository::update(conn, user_id, update_data)
}

pub fn soft_delete_user(conn: &mut PgConnection, user_id: Uuid) -> Result<usize, AuthError> {
    repository::soft_delete(conn, user_id)
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<DbUser>, AuthError> {
    repository::find_all(conn)
}
