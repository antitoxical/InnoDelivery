use crate::dto::user_dto::UpdateUser;
use crate::models::user::{NewUser as DbNewUser, User as DbUser};
use crate::schema::users::{
    self,
    dsl::{is_deleted, users as all_users},
};
use crate::services::auth_service::AuthError;
use diesel::prelude::*;
use uuid::Uuid;

pub fn find_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<DbUser, AuthError> {
    all_users
        .find(user_id)
        .select(DbUser::as_select())
        .first::<DbUser>(conn)
        .map_err(AuthError::from)
}

pub fn find_by_phone(conn: &mut PgConnection, phone: &str) -> Result<DbUser, AuthError> {
    all_users
        .filter(users::phone_number.eq(phone))
        .select(DbUser::as_select())
        .first::<DbUser>(conn)
        .map_err(AuthError::from)
}

pub fn find_all(conn: &mut PgConnection) -> Result<Vec<DbUser>, AuthError> {
    all_users
        .filter(is_deleted.eq(false))
        .select(DbUser::as_select())
        .load::<DbUser>(conn)
        .map_err(AuthError::from)
}

pub fn create(conn: &mut PgConnection, new_user: DbNewUser) -> Result<DbUser, AuthError> {
    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(DbUser::as_returning())
        .get_result::<DbUser>(conn)
        .map_err(AuthError::from)
}

pub fn update(
    conn: &mut PgConnection,
    user_id: Uuid,
    update_data: UpdateUser,
) -> Result<DbUser, AuthError> {
    diesel::update(all_users.find(user_id))
        .set(&update_data)
        .get_result::<DbUser>(conn)
        .map_err(AuthError::from)
}

pub fn soft_delete(conn: &mut PgConnection, user_id: Uuid) -> Result<usize, AuthError> {
    diesel::update(all_users.find(user_id))
        .set(is_deleted.eq(true))
        .execute(conn)
        .map_err(AuthError::from)
}
