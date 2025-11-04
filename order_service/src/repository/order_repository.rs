use crate::models::{NewOrder, NewOrderProduct, Order, OrderStatus};
use crate::schema::{order_products, orders, products};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use uuid::Uuid;

#[derive(Debug)]
pub struct OrderProductData {
    pub product_id: Uuid,
    pub quantity: i32,
}

pub fn create_order(
    conn: &mut PgConnection,
    new_order: NewOrder,
    products: Vec<OrderProductData>,
) -> Result<Order, DieselError> {
    conn.transaction(|connection| {
        let created_order = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result::<Order>(connection)?;

        let order_products_to_insert: Vec<NewOrderProduct> = products
            .into_iter()
            .map(|p| NewOrderProduct {
                order_id: created_order.id,
                product_id: p.product_id,
                quantity: p.quantity,
            })
            .collect();

        diesel::insert_into(order_products::table)
            .values(&order_products_to_insert)
            .execute(connection)?;

        Ok(created_order)
    })
}

pub fn find_order_by_id(conn: &mut PgConnection, order_uuid: Uuid) -> Result<Order, DieselError> {
    orders::table.find(order_uuid).first(conn)
}

pub fn update_order_status(
    conn: &mut PgConnection,
    order_uuid: Uuid,
    new_status: OrderStatus,
) -> Result<Order, DieselError> {
    diesel::update(orders::table.find(order_uuid))
        .set(orders::status.eq(new_status))
        .get_result(conn)
}

pub fn update_order_address(
    conn: &mut PgConnection,
    order_uuid: Uuid,
    new_address: &str,
) -> Result<Order, DieselError> {
    diesel::update(orders::table.find(order_uuid))
        .set(orders::delivery_address.eq(new_address))
        .get_result(conn)
}

pub fn list_orders_by_user(
    conn: &mut PgConnection,
    user_uuid: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Order>, DieselError> {
    orders::table
        .filter(orders::user_id.eq(user_uuid))
        .order(orders::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<Order>(conn)
}

pub fn find_missing_product_ids(
    conn: &mut PgConnection,
    product_ids: &[Uuid],
) -> Result<Vec<Uuid>, DieselError> {
    if product_ids.is_empty() {
        return Ok(vec![]);
    }

    let existing: Vec<Uuid> = products::table
        .filter(products::id.eq_any(product_ids))
        .select(products::id)
        .load(conn)?;

    let existing_set: std::collections::HashSet<Uuid> = existing.into_iter().collect();

    Ok(product_ids
        .iter()
        .cloned()
        .filter(|id| !existing_set.contains(id))
        .collect())
}

pub fn find_expired_pending_orders(
    conn: &mut PgConnection,
    created_before: chrono::NaiveDateTime,
) -> Result<Vec<Order>, DieselError> {
    use crate::schema::orders::dsl::{created_at, status};

    crate::schema::orders::table
        .filter(status.eq(OrderStatus::PendingCarrier))
        .filter(created_at.lt(created_before))
        .load::<Order>(conn)
}

pub fn update_order_status_and_courier(
    conn: &mut PgConnection,
    order_uuid: Uuid,
    new_status: OrderStatus,
    courier_id: Option<Uuid>,
) -> Result<Order, DieselError> {
    diesel::update(orders::table.find(order_uuid))
        .set((
            orders::status.eq(new_status),
            orders::courier_id.eq(courier_id),
            orders::updated_at.eq(chrono::Utc::now().naive_utc()),
        ))
        .get_result(conn)
}

pub fn rate_order_in_window(
    conn: &mut PgConnection,
    order_uuid: Uuid,
    rating: f32,
    rating_window_minutes: i32,
) -> Result<Order, DieselError> {
    use crate::schema::orders;

    if !(1.0..=5.0).contains(&rating) {
        return Err(DieselError::QueryBuilderError(
            "Invalid rating value".into(),
        ));
    }

    let rating_window = chrono::Duration::minutes(i64::from(rating_window_minutes));
    let max_rating_time = chrono::Utc::now().naive_utc() - rating_window;

    diesel::update(
        orders::table
            .find(order_uuid)
            .filter(orders::status.eq(OrderStatus::Finished))
            .filter(orders::updated_at.gt(max_rating_time))
            .filter(orders::rating.is_null()),
    )
    .set((
        orders::rating.eq(rating),
        orders::updated_at.eq(chrono::Utc::now().naive_utc()),
    ))
    .get_result(conn)
}
