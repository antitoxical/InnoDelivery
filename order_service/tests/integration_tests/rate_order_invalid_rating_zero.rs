use axum::http::{Request, StatusCode};
use dotenvy::dotenv;
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
async fn rate_order_invalid_rating_zero() {
    dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (app, pool, _server) = helpers::setup_test_app(&*db_url).await;
    let user_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: None,
            delivery_address: "addr rate invalid".to_string(),
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

    let mutation = format!(
        r#"
        mutation RateOrder {{
            rateOrder(
                id: "{}",
                rating: 0.0,
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

    assert!(json_body["data"].is_null());
    let errors = json_body["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);
    let error_message = errors[0]["message"].as_str().unwrap();
    assert!(error_message.contains("Invalid rating"));
}
