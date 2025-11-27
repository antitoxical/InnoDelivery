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
async fn test_orders_by_user_pagination() {
    dotenvy::dotenv().ok();
    let db_url =
        std::env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");
    let (app, pool, _server) = helpers::setup_test_app(&db_url).await;

    let user_id = Uuid::new_v4();

    {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);

        for i in 0..10 {
            let new_order = NewOrder {
                user_id,
                courier_id: None,
                delivery_address: format!("addr {}", i),
                status: OrderStatus::Finished,
            };
            order_repository::create_order(
                &mut conn,
                new_order,
                vec![OrderProductData {
                    product_id: prod_id,
                    quantity: 1,
                }],
            )
            .expect("create order");
        }
    }

    let query_limit = format!(
        r#"
        query {{
            ordersByUser(userId: "{}", limit: 5, offset: 0) {{
                deliveryAddress
            }}
        }}
        "#,
        user_id
    );

    let req = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": query_limit }).to_string())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let list = json["data"]["ordersByUser"].as_array().unwrap();
    assert_eq!(list.len(), 5);

    let query_offset = format!(
        r#"
        query {{
            ordersByUser(userId: "{}", limit: 5, offset: 5) {{
                deliveryAddress
            }}
        }}
        "#,
        user_id
    );

    let req = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": query_offset }).to_string())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let list = json["data"]["ordersByUser"].as_array().unwrap();
    assert_eq!(list.len(), 5);
}
