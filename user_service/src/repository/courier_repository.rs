use crate::models::courier::{Courier, CourierStatus, NewCourier};
use crate::models::user::User;
use crate::schema::{couriers, users};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::PgConnection;
use uuid::Uuid;

pub fn create(conn: &mut PgConnection, new_courier: &NewCourier) -> Result<Courier, DieselError> {
    diesel::insert_into(couriers::table)
        .values(new_courier)
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
        .get_result::<Courier>(conn)
}

