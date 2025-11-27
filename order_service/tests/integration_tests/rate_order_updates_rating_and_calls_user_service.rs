use axum::http::{Request, StatusCode};
use httpmock::Method::POST;
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fakers;
use crate::helpers;

#[tokio::test]
async fn rate_order_updates_rating_and_calls_user_service() {
    dotenvy::dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");

    let (app, pool, server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();
    let courier_id = Uuid::new_v4();
    let rating_value = 5.0f32;
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
            .json_body(json!({"courier_id": courier_id, "rating": rating_value}));
        then.status(200);
    });

    let mutation = format!(
        r#"
        mutation RateOrder {{
            rateOrder(
                id: "{}",
                rating: {},
                userId: "{}",
                ratingWindowMinutes: 1440
            ) {{
                id
                rating
            }}
        }}
        "#,
        order.id, rating_value, user_id
    );

    let request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": mutation }).to_string())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    assert_eq!(
        status,
        StatusCode::OK,
        "Expected status OK, but got {}. Body: {}",
        status,
        body_str
    );

    let json_body: serde_json::Value =
        serde_json::from_str(&body_str).expect("Failed to parse response body as JSON");

    if let Some(errors) = json_body.get("errors") {
        panic!("GraphQL query returned errors: {}", errors);
    }

    let data = &json_body["data"]["rateOrder"];
    assert_eq!(data["id"], order.id.to_string());
    assert_eq!(data["rating"], rating_value);
    rating_mock.assert();
}
