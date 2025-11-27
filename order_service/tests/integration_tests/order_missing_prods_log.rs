use crate::helpers;
use order_service::repository::order_repository::OrderProductData;
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_service_create_order_missing_products_log() {
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (_app, pool, _server) = helpers::setup_test_app(&db_url).await;
    let user_id = Uuid::new_v4();

    let missing_id = Uuid::new_v4();

    let result = order_service::services::order_service::create_order(
        &pool,
        "http://dummy",
        user_id,
        "addr",
        vec![OrderProductData {
            product_id: missing_id,
            quantity: 1,
        }],
    )
    .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown product ids"));
}
