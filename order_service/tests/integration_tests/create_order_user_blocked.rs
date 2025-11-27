use crate::fakers;
use crate::helpers;
use axum::http::{Request, StatusCode};
use httpmock::Method::GET;
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn create_order_user_blocked() {
    dotenvy::dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (app, pool, server) = helpers::setup_test_app(&db_url).await;

    let prod_id = {
        let mut conn = pool.get().expect("pool conn");
        fakers::insert_product(&mut conn)
    };
    let user_id = Uuid::new_v4();

    let user_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(200).body(r#"{"is_blocked":true}"#);
    });

    let mutation = format!(
        r#"
        mutation CreateOrder {{
            createOrder(
                userId: "{}",
                deliveryAddress: "addr blocked",
                products: [ {{ productId: "{}", quantity: 1 }} ]
            ) {{
                id
            }}
        }}
        "#,
        user_id, prod_id
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
        "Expected status OK. Body: {}",
        body_str
    );

    let json_body: serde_json::Value =
        serde_json::from_str(&body_str).expect("Failed to parse response body as JSON");

    assert!(json_body["data"].is_null());
    let errors = json_body["errors"]
        .as_array()
        .expect("Expected 'errors' array");
    assert_eq!(errors.len(), 1);

    let error_message = errors[0]["message"].as_str().unwrap();
    assert!(error_message.contains("User is blocked"));

    user_mock.assert();
}
