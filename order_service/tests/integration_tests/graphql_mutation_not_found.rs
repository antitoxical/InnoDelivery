use crate::helpers;
use axum::http::Request;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_mutations_order_not_found() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (app, _pool, _server) = helpers::setup_test_app(&db_url).await;

    let random_id = Uuid::new_v4();

    let update_query = format!(
        r#"mutation {{ updateOrderAddress(orderId: "{}", deliveryAddress: "new") {{ id }} }}"#,
        random_id
    );
    let req = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": update_query }).to_string())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Record not found"));

    let complete_query = format!(
        r#"mutation {{ completeOrder(id: "{}") {{ id }} }}"#,
        random_id
    );
    let req = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": complete_query }).to_string())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Record not found"));

    let cancel_query = format!(
        r#"mutation {{ cancelOrder(id: "{}") {{ id }} }}"#,
        random_id
    );
    let req = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": cancel_query }).to_string())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["errors"][0]["message"]
        .as_str()
        .unwrap()
        .contains("Record not found"));
}
