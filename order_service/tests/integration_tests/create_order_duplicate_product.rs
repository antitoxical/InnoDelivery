use crate::fakers;
use crate::helpers;
use axum::http::{Request, StatusCode};
use httpmock::Method::{GET, POST};
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn create_order_duplicate_product_db_error() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (app, pool, server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();
    let prod_id = {
        let mut conn = pool.get().expect("conn");
        fakers::insert_product(&mut conn)
    };

    let user_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(200).body(r#"{"is_blocked":false}"#);
    });

    let assign_mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(204);
    });

    let mutation = format!(
        r#"
        mutation CreateOrder {{
            createOrder(
                userId: "{}",
                deliveryAddress: "addr duplicate",
                products: [
                    {{ productId: "{}", quantity: 1 }},
                    {{ productId: "{}", quantity: 2 }}
                ]
            ) {{
                id
            }}
        }}
        "#,
        user_id, prod_id, prod_id
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
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"].is_null());
    let errors = json["errors"].as_array().unwrap();
    assert!(!errors.is_empty());

    let error = &errors[0];
    let msg = error["message"].as_str().unwrap();

    assert!(
        msg.contains("Database error")
            || msg.contains("duplicate key")
            || msg.contains("unique constraint")
    );

    user_mock.assert();
    assign_mock.assert();
}
