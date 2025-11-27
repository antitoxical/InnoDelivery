use axum::http::{Request, StatusCode};
use httpmock::Method::GET;
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

use crate::helpers;

#[tokio::test]
async fn create_order_with_missing_product() {
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (app, _pool, server) = helpers::setup_test_app(&*db_url).await;
    let user_id = Uuid::new_v4();
    let non_existent_prod_id = Uuid::new_v4();

    let user_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(200).body(r#"{"is_blocked":false}"#);
    });

    let mutation = format!(
        r#"
        mutation CreateOrder {{
            createOrder(
                userId: "{}",
                deliveryAddress: "addr missing",
                products: [ {{ productId: "{}", quantity: 1 }} ]
            ) {{
                id
            }}
        }}
        "#,
        user_id, non_existent_prod_id
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
    assert!(error_message.contains("Invalid input: Unknown product ids"));
    assert!(error_message.contains(&non_existent_prod_id.to_string()));

    user_mock.assert_hits(0);
}
