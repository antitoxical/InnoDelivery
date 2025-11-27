use axum::http::{Request, StatusCode};
use dotenvy::dotenv;
use order_service::{
    models::{NewOrder, OrderStatus},
    repository::order_repository::{self, OrderProductData},
};
use serde_json::json;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;

use crate::fakers;
use crate::helpers;

#[tokio::test]
async fn list_orders_by_user_with_orders() {
    dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (app, pool, _server) = helpers::setup_test_app(&*db_url).await;
    let user_id = Uuid::new_v4();

    {
        let mut conn = pool.get().expect("pool conn");
        let prod_id = fakers::insert_product(&mut conn);
        for i in 0..3 {
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

    let query = format!(
        r#"
        query {{
            ordersByUser(userId: "{}", limit: 10, offset: 0) {{
                id
                status
            }}
        }}
        "#,
        user_id
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

    let data = &json_body["data"]["ordersByUser"];
    assert_eq!(data.as_array().unwrap().len(), 3);
}
