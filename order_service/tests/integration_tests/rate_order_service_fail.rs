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
async fn rate_order_courier_service_fail() {
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");
    let (app, pool, server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();
    let courier_id = Uuid::new_v4();

    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: Some(courier_id),
            delivery_address: "addr rate".to_string(),
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

    let rating_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/internal/couriers/set_rating")
            .json_body(json!({"courier_id": courier_id, "rating": 4.0}));
        then.status(500);
    });

    let mutation = format!(
        r#"
        mutation RateOrder {{
            rateOrder(
                id: "{}",
                rating: 4.0,
                userId: "{}",
                ratingWindowMinutes: 1440
            ) {{
                id
                rating
            }}
        }}
        "#,
        order.id, user_id
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

    let data = &json_body["data"]["rateOrder"];
    assert_eq!(data["id"], order.id.to_string());
    assert_eq!(data["rating"], 4.0);

    rating_mock.assert();
}
