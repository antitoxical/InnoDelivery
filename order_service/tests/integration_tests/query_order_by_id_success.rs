use crate::fakers;
use crate::helpers;
use axum::http::{Request, StatusCode};
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn query_order_by_id_success() {
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");
    let (app, pool, _server) = helpers::setup_test_app(&db_url).await;

    let order = {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        let new_order = NewOrder {
            user_id: Uuid::new_v4(),
            courier_id: None,
            delivery_address: "addr query".to_string(),
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

    let query = format!(
        r#"
        query {{
            orderById(id: "{}") {{
                id
                status
                deliveryAddress
            }}
        }}
        "#,
        order.id
    );

    let request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": query }).to_string())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_body: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let data = &json_body["data"]["orderById"];
    assert_eq!(data["id"], order.id.to_string());
    assert_eq!(data["status"], "DRAFT");
    assert_eq!(data["deliveryAddress"], "addr query");
}
