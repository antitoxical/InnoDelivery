use crate::fakers;
use crate::helpers;
use axum::http::{Request, StatusCode};
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_cancel_order_mutation() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (app, pool, _server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: None,
            delivery_address: "addr to cancel".to_string(),
            status: OrderStatus::PendingCarrier,
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
        mutation {{
            cancelOrder(id: "{}") {{
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
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let data = &json["data"]["cancelOrder"];
    assert_eq!(data["id"], order.id.to_string());
    assert_eq!(data["status"], "CANCELLED");
}
