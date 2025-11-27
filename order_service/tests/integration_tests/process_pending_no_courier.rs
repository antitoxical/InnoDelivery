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
async fn process_pending_orders_cancels_when_no_courier() {
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
            delivery_address: "addr pending cancel".to_string(),
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
        .expect("create order failed");

        order_id = order.id;

        diesel::update(order_service::schema::orders::table.find(order_id))
            .set(
                order_service::schema::orders::created_at
                    .eq(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(10_000)),
            )
            .execute(&mut conn)
            .expect("update created_at failed");
    }

    let assign_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(204);
    });

    let processed = order_service::services::order_service::process_pending_orders(
        &pool,
        server.base_url().as_str(),
        1,
    )
    .await
    .expect("process_pending_orders failed");

    assert_eq!(processed, 1);

    let updated_order = {
        let mut conn = pool.get().expect("pool conn verification");
        order_repository::find_order_by_id(&mut conn, order_id).expect("find failed")
    };

    assert_eq!(updated_order.status, OrderStatus::Cancelled);
    assign_mock.assert();
}
