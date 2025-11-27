use crate::helpers;
use dotenvy::dotenv;
use order_service::models::{NewOrder, OrderStatus};
use order_service::repository::order_repository;
use order_service::repository::order_repository::OrderProductData;
use order_service::services::order_service::rate_order;
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn rate_order_user_service_network_error() {
    dotenv().ok();
    let test_url = "http://127.0.0.1:1";
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (_app, pool, _server) = helpers::setup_test_app(&*db_url).await;
    let user_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = crate::fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: Some(Uuid::new_v4()),
            delivery_address: "addr rate network error".to_string(),
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

    let result = rate_order(&pool, test_url, order.id, user_id, 5.0, 1440).await;

    assert!(result.is_ok());
}
