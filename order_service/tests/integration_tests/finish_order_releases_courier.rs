use axum::http::{Request, StatusCode};
use httpmock::Method::POST;
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
async fn finish_order_releases_courier() {
    let (app, pool, server) = helpers::setup_test_app().await;

    let user_id = Uuid::new_v4();
    let courier_id = Uuid::new_v4();
    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id,
            courier_id: Some(courier_id),
            delivery_address: "addr finish".to_string(),
            status: OrderStatus::InProgress,
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

    let release_mock = server.mock(|when, then| {
        when.method(POST)
            .path("/internal/couriers/free")
            .json_body(json!({"courier_id": courier_id}));
        then.status(200);
    });

    let mutation = format!(
        r#"
        mutation FinishOrder {{
            finishOrder(id: "{}") {{
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
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let data = &json_body["data"]["finishOrder"];
    assert_eq!(data["id"], order.id.to_string());
    assert_eq!(data["status"], "FINISHED");
    release_mock.assert();
}
