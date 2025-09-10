use crate::schema::couriers;
use crate::models::user::User;
use diesel::prelude::*;
use diesel::{AsExpression, FromSqlRow};
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output};
use diesel::sql_types::Text;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
pub enum CourierStatus {
    Free,
    Busy,
}

impl ToSql<Text, diesel::pg::Pg> for CourierStatus
where
    String: ToSql<Text, diesel::pg::Pg>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::pg::Pg>) -> serialize::Result {
        let s = match self {
            CourierStatus::Free => "free",
            CourierStatus::Busy => "busy",
        };
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, diesel::pg::Pg> for CourierStatus
where
    String: FromSql<Text, diesel::pg::Pg>,
{
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, diesel::pg::Pg>>::from_sql(bytes)?;
        match s.as_str() {
            "free" => Ok(CourierStatus::Free),
            "busy" => Ok(CourierStatus::Busy),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Identifiable, Associations, Selectable, Debug, Serialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = couriers)]
pub struct Courier {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: CourierStatus,
    pub rating: f32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = couriers)]
pub struct NewCourier {
    pub user_id: Uuid,
    pub status: CourierStatus,
}

