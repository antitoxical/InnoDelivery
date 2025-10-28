// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "order_status"))]
    pub struct OrderStatus;
}

diesel::table! {
    order_products (id) {
        id -> Uuid,
        order_id -> Uuid,
        product_id -> Uuid,
        quantity -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::OrderStatus;

    orders (id) {
        id -> Uuid,
        user_id -> Uuid,
        courier_id -> Nullable<Uuid>,
        delivery_address -> Varchar,
        status -> OrderStatus,
        rating -> Nullable<Float4>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        product_type -> Varchar,
        product_name -> Varchar,
        restaurant -> Varchar,
        price -> Float4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(order_products -> orders (order_id));
diesel::joinable!(order_products -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(order_products, orders, products,);
