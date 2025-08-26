// src/services/user_service.rs

use diesel::prelude::*;
use crate::models::user::User as DbUser;
use crate::schema::users;
use uuid::Uuid;
use tracing;
use crate::dto::user_dto::UpdateUser;
use crate::services::auth_service::AuthError;
use crate::schema::users::dsl::{users as all_users, is_deleted};


pub fn get_user_profile(conn: &mut PgConnection, user_id: Uuid) -> Result<DbUser, AuthError> {
    tracing::debug!("Search for a user by ID: {}", user_id);
    users::table
        .find(user_id)
        .select(DbUser::as_select())
        .first::<DbUser>(conn)
        .map_err(|e| {
            tracing::error!("Failed to find a user with ID {}: {:?}", user_id, e);
            AuthError::from(e)
        })
}


pub fn update_user_profile(
    conn: &mut PgConnection,
    user_id: Uuid,
    update_data: UpdateUser,
) -> Result<DbUser, AuthError> {
    tracing::debug!("Profile update for ID user: {}", user_id);

    diesel::update(users::table.find(user_id))
        .set(&update_data)
        .get_result::<DbUser>(conn)
        .map_err(|e| {
            tracing::error!("Error when updating user with ID {}: {:?}", user_id, e);
            if let diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) = e {
                return AuthError::DatabaseError("Email or phone number are already busy.".to_string());
            }
            AuthError::from(e)
        })
}

pub fn soft_delete_user(conn: &mut PgConnection, user_id: Uuid) -> Result<usize, AuthError> {
    tracing::debug!("Soft removal of the user with ID: {}", user_id);
    diesel::update(users::table.find(user_id))
        .set(is_deleted.eq(true))
        .execute(conn)
        .map_err(|e| {
            tracing::error!("Error with mild removal of the user with ID {}: {:?}", user_id, e);
            AuthError::from(e)
        })
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<DbUser>, AuthError> {
    tracing::debug!("Obtaining a list of all users");
    all_users
        .filter(is_deleted.eq(false))
        .select(DbUser::as_select())
        .load::<DbUser>(conn)
        .map_err(|e| {
            tracing::error!("Error when receiving a user list: {:?}", e);
            AuthError::from(e)
        })
}
