use crate::models::courier::{Courier, CourierStatus, NewCourier};
use crate::models::user::User;
use crate::schema::{couriers, users};
use diesel::PgConnection;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use uuid::Uuid;

pub fn create(conn: &mut PgConnection, new_courier: &NewCourier) -> Result<Courier, DieselError> {
    diesel::insert_into(couriers::table)
        .values(new_courier)
        .returning(Courier::as_returning())
        .get_result(conn)
}

pub fn find_courier_by_user_id(
    conn: &mut PgConnection,
    uid: Uuid,
) -> Result<(User, Courier), DieselError> {
    users::table
        .inner_join(couriers::table.on(couriers::user_id.eq(users::id)))
        .filter(users::id.eq(uid))
        .select((User::as_select(), Courier::as_select()))
        .first::<(User, Courier)>(conn)
}

pub fn update_status_by_user_id(
    conn: &mut PgConnection,
    uid: Uuid,
    new_status: CourierStatus,
) -> Result<Courier, DieselError> {
    diesel::update(couriers::table.filter(couriers::user_id.eq(uid)))
        .set(couriers::status.eq(new_status))
        .returning(Courier::as_returning())
        .get_result::<Courier>(conn)
}

pub fn get_all(conn: &mut PgConnection) -> Result<Vec<(User, Courier)>, DieselError> {
    users::table
        .inner_join(couriers::table.on(couriers::user_id.eq(users::id)))
        .select((User::as_select(), Courier::as_select()))
        .load::<(User, Courier)>(conn)
}

pub fn set_blocked_status(
    conn: &mut PgConnection,
    courier_user_id: Uuid,
    blocked: bool,
) -> Result<Courier, DieselError> {
    diesel::update(couriers::table.filter(couriers::user_id.eq(courier_user_id)))
        .set(couriers::is_blocked.eq(blocked))
        .returning(Courier::as_returning())
        .get_result::<Courier>(conn)
}

pub fn soft_delete(conn: &mut PgConnection, courier_user_id: Uuid) -> Result<usize, DieselError> {
    diesel::update(couriers::table.filter(couriers::user_id.eq(courier_user_id)))
        .set(couriers::is_deleted.eq(true))
        .execute(conn)
}
