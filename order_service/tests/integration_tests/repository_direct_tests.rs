use crate::fakers;
use crate::helpers;
use order_service::models::{NewOrder, OrderStatus};
use order_service::repository::order_repository::{self, OrderProductData};
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_repository_find_missing_ids_empty() {
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (_app, pool, _) = helpers::setup_test_app(&db_url).await;

    let mut conn = pool.get().expect("pool conn");

    let result = order_repository::find_missing_product_ids(&mut conn, &[]);

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_repository_rate_order_invalid_value() {
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (_app, pool, _) = helpers::setup_test_app(&db_url).await;

    let mut conn = pool.get().expect("pool conn");
    let prod_id = fakers::insert_product(&mut conn);

    let order = order_repository::create_order(
        &mut conn,
        NewOrder {
            user_id: Uuid::new_v4(),
            courier_id: None,
            delivery_address: "addr".to_string(),
            status: OrderStatus::Finished,
        },
        vec![OrderProductData {
            product_id: prod_id,
            quantity: 1,
        }],
    )
    .unwrap();

    let result = order_repository::rate_order_in_window(&mut conn, order.id, 6.0, 1440);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(err.to_string().contains("Invalid rating value"));
}
