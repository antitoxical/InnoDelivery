use axum::http::{Request, StatusCode};
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fakers;
use crate::helpers;

#[tokio::test]
async fn update_order_address_and_cancel() {
    let (app, pool, _server) = helpers::setup_test_app().await;

    let user_id = Uuid::new_v4();
    let new_address = "addr new".to_string();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: None,
            delivery_address: "addr old".to_string(),
            status: OrderStatus::Draft,
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

    let update_mutation = format!(
        r#"
        mutation UpdateAddress {{
            updateOrderAddress(
                orderId: "{}",
                deliveryAddress: "{}"
            ) {{
                id
                deliveryAddress
            }}
        }}
        "#,
        order.id, new_address
    );

    let update_request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": update_mutation }).to_string())
        .unwrap();

    let update_response = app.clone().oneshot(update_request).await.unwrap();

    assert_eq!(update_response.status(), StatusCode::OK);
    let update_body_bytes = axum::body::to_bytes(update_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let update_json: serde_json::Value = serde_json::from_slice(&update_body_bytes).unwrap();
    let update_data = &update_json["data"]["updateOrderAddress"];
    assert_eq!(update_data["id"], order.id.to_string());
    assert_eq!(update_data["deliveryAddress"], new_address);

    let cancel_mutation = format!(
        r#"
        mutation CancelOrder {{
            cancelOrder(id: "{}") {{
                id
                status
            }}
        }}
        "#,
        order.id
    );

    let cancel_request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": cancel_mutation }).to_string())
        .unwrap();

    let cancel_response = app.oneshot(cancel_request).await.unwrap();

    assert_eq!(cancel_response.status(), StatusCode::OK);
    let cancel_body_bytes = axum::body::to_bytes(cancel_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let cancel_json: serde_json::Value = serde_json::from_slice(&cancel_body_bytes).unwrap();
    let cancel_data = &cancel_json["data"]["cancelOrder"];
    assert_eq!(cancel_data["id"], order.id.to_string());
    assert_eq!(cancel_data["status"], "CANCELLED");
}
