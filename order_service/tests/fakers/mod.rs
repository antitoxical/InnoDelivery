use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

pub fn insert_product(conn: &mut PgConnection) -> Uuid {
    use order_service::schema::products::dsl::*;
    let new_id = Uuid::new_v4();
    diesel::insert_into(products)
        .values((
            id.eq(new_id),
            product_type.eq("Food"),
            product_name.eq("Pizza"),
            restaurant.eq("Restaurant"),
            price.eq(9.99f32),
            created_at.eq(Utc::now().naive_utc()),
            updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)
        .expect("Failed to insert product");
    new_id
}
