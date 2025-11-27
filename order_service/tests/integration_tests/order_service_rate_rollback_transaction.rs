use crate::helpers;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenvy::dotenv;
use order_service::models::{NewOrder, OrderStatus};
use order_service::repository::order_repository::{self, OrderProductData};
use order_service::services::order_service::{rate_order, OrderServiceError};
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn rate_order_rollback_transaction_error() {
    dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (_app, pool, server) = helpers::setup_test_app(&*db_url).await;

    let user_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = crate::fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: None,
            delivery_address: "addr rate rollback".to_string(),
            status: OrderStatus::Finished,
        };
        order_repository::create_order(
            &mut conn,
            new_order,
            vec![OrderProductData {
                product_id: prod_id,
                quantity: 1,
            }],
        )
        .expect("create order")
    };

    {
        let mut conn = pool.get().expect("pool conn for update");
        diesel::update(order_service::schema::orders::table.find(order.id))
            .set((
                order_service::schema::orders::created_at
                    .eq(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(10_000)),
                order_service::schema::orders::updated_at
                    .eq(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(10_000)),
            ))
            .execute(&mut conn)
            .expect("update timestamps failed");
    }

    let result = rate_order(&pool, &server.base_url(), order.id, user_id, 5.0, 1).await;

    match result {
        Err(OrderServiceError::RatingWindowExpired) => (),
        _ => panic!("Expected RatingWindowExpired, got {:?}", result),
    }
}
