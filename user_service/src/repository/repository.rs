use crate::dto::user_dto::UpdateUser as UpdateUserDto;
use crate::models::user::{NewUser as DbNewUser, User as DbUser};
use crate::schema::users::dsl::*;
use crate::schema::users::{
    self,
    dsl::{is_deleted, users as all_users},
};
use crate::services::auth_service::AuthError;
use diesel::prelude::*;
use uuid::Uuid;

pub fn create(
    conn: &mut PgConnection,
    new_user: &DbNewUser,
) -> Result<DbUser, diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::insert_into(users::table)
            .values(new_user)
            .returning(DbUser::as_returning())
            .get_result(conn)
    })
}

pub fn find_by_phone(
    conn: &mut PgConnection,
    phone: &str,
) -> Result<DbUser, diesel::result::Error> {
    all_users
        .filter(users::phone_number.eq(phone))
        .filter(is_deleted.eq(false))
        .select(DbUser::as_select())
        .first(conn)
}

pub fn find_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<DbUser, diesel::result::Error> {
    all_users
        .find(user_id)
        .select(DbUser::as_select())
        .first(conn)
}

pub fn get_all_users(conn: &mut PgConnection) -> Result<Vec<DbUser>, diesel::result::Error> {
    all_users
        .filter(is_deleted.eq(false))
        .select(DbUser::as_select())
        .load::<DbUser>(conn)
}
pub fn update(
    conn: &mut PgConnection,
    user_id: Uuid,
    user_data: &UpdateUserDto,
) -> Result<DbUser, diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::update(all_users.find(user_id))
            .set(user_data)
            .returning(DbUser::as_returning())
            .get_result(conn)
    })
}

pub fn soft_delete_user(
    conn: &mut PgConnection,
    user_id: Uuid,
) -> Result<usize, diesel::result::Error> {
    conn.transaction(|conn| {
        diesel::update(all_users.find(user_id))
            .set(is_deleted.eq(true))
            .execute(conn)
    })
}
