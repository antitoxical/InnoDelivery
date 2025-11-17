use crate::fakers;
use crate::helpers;
use diesel::prelude::*;
use dotenvy::dotenv;
use httpmock::Method::POST;
use order_service::models::{NewOrder, OrderStatus};
use order_service::repository::order_repository::{self, OrderProductData};
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn process_pending_orders_assigns_or_cancels() {
    dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (_app, pool, server) = helpers::setup_test_app(&db_url).await;
    let user_id = Uuid::new_v4();
    let order_id;

    {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);

        let new_order = NewOrder {
            user_id,
            courier_id: None,
            delivery_address: "addr pending".to_string(),
            status: OrderStatus::PendingCarrier,
        };
        let order = order_repository::create_order(
            &mut conn,
            new_order,
            vec![OrderProductData {
                product_id: prod_id,
                quantity: 1,
            }],
        )
        .expect("create order failed in test setup");

        order_id = order.id;

        diesel::update(order_service::schema::orders::table.find(order_id))
            .set(
                order_service::schema::orders::created_at
                    .eq(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(10_000)),
            )
            .execute(&mut conn)
            .expect("update created_at failed");
    }
    let courier_id = Uuid::new_v4();
    let assign_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .body(format!("{{\"courier_id\":\"{}\"}}", courier_id));
    });

    let processed = order_service::services::order_service::process_pending_orders(
        &pool,
        server.base_url().as_str(),
        1,
    )
    .await
    .expect("process_pending_orders failed");

    assert!(processed >= 1);

    let updated_order = {
        let mut conn = pool.get().expect("pool conn for verification");
        order_repository::find_order_by_id(&mut conn, order_id)
            .expect("find order for verification failed")
    };
    assert!(
        updated_order.status == OrderStatus::InProgress
            || updated_order.status == OrderStatus::Cancelled
    );

    if updated_order.status == OrderStatus::InProgress {
        assign_mock.assert();
    } else {
        assign_mock.assert_hits(1);
    }
}
