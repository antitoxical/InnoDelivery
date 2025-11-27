use crate::helpers;
use httpmock::Method::{GET, POST};
use order_service::repository::order_repository::OrderProductData;
use serde_json::json;
use std::env;
use tracing::Level;
use uuid::Uuid;

#[tokio::test]
async fn test_logging_coverage_create_order() {
    dotenvy::dotenv().ok();

    let _ = tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_test_writer()
        .try_init();

    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (_app, pool, server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();
    let courier_id = Uuid::new_v4();
    let prod_id = {
        let mut conn = pool.get().expect("pool");
        crate::fakers::insert_product(&mut conn)
    };

    server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .header("content-type", "application/json")
            .body(json!({"courier_id": courier_id}).to_string());
    });

    server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/busy");
        then.status(200);
    });

    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(200).body(r#"{"is_blocked":false}"#);
    });

    let result = order_service::services::order_service::create_order(
        &pool,
        &server.base_url(),
        user_id,
        "addr log test",
        vec![OrderProductData {
            product_id: prod_id,
            quantity: 1,
        }],
    )
    .await;

    assert!(result.is_ok());
}
