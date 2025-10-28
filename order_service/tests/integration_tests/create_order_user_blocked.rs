use axum::http::{Request, StatusCode};
use httpmock::Method::GET;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fakers;
use crate::helpers;

#[tokio::test]
async fn create_order_user_blocked() {
    let (app, pool, server) = helpers::setup_test_app().await;

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

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json_body["data"].is_null());
    let errors = json_body["errors"].as_array().unwrap();
    assert_eq!(errors.len(), 1);

    let error_message = errors[0]["message"].as_str().unwrap();
    assert!(error_message.contains("User is blocked"));

    user_mock.assert();
}
