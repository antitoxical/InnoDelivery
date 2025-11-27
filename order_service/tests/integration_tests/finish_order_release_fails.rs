use crate::fakers;
use crate::helpers;
use axum::http::{Request, StatusCode};
use httpmock::Method::POST;
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn finish_order_release_fails() {
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");
    let (app, pool, server) = helpers::setup_test_app(&db_url).await;

    let courier_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id: Uuid::new_v4(),
            courier_id: Some(courier_id),
            delivery_address: "addr finish fail".to_string(),
            status: OrderStatus::InProgress,
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

    let release_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/internal/couriers/free")
            .json_body(json!({"courier_id": courier_id}));
        then.status(500);
    });

    let mutation = format!(
        r#"
        mutation CompleteOrder {{
            completeOrder(id: "{}") {{
                id
                status
            }}
        }}
        "#,
        order.id
    );

    let request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": mutation }).to_string())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json_body["data"].is_null());
    let errors = json_body["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    release_mock.assert();
}
