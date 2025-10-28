use axum::http::{Request, StatusCode};
use httpmock::Method::{GET, POST};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fakers;
use crate::helpers;

#[tokio::test]
async fn create_order_courier_busy_fails_sets_pending() {
    let (app, pool, server) = helpers::setup_test_app().await;

    let prod_id = {
        let mut conn = pool.get().expect("pool conn");
        fakers::insert_product(&mut conn)
    };
    let user_id = Uuid::new_v4();

    let user_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(200).body(r#"{"is_blocked":false}"#);
    });
    let courier_id = Uuid::new_v4();
    let assign_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .body(json!({"courier_id": courier_id}).to_string());
    });
    let busy_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/busy");
        then.status(500);
    });
    let release_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/free");
        then.status(200);
    });

    let mutation = format!(
        r#"
        mutation CreateOrder {{
            createOrder(
                userId: "{}",
                deliveryAddress: "addr busy fail",
                products: [ {{ productId: "{}", quantity: 1 }} ]
            ) {{
                id
                status
                courierId
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

    let data = &json_body["data"]["createOrder"];
    assert_eq!(data["status"], "PENDING_CARRIER");
    assert!(data["courierId"].is_null());

    user_mock.assert();
    assign_mock.assert();
    busy_mock.assert();
    release_mock.assert();
}
