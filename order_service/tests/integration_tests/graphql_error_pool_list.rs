use axum::http::{Request, StatusCode};
use axum::{Extension, Router};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use order_service::graphql::{graphql_handler, AppSchema, MutationRoot, QueryRoot};
use serde_json::json;
use std::env;
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_graphql_pool_exhaustion_orders_list() {
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");

    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_millis(100))
        .build(manager)
        .expect("Failed to create constrained pool");

    let _guard = pool.get().expect("Failed to get the only connection");

    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pool.clone())
        .finish();

    let app = Router::new()
        .route("/", axum::routing::post(graphql_handler))
        .layer(Extension(schema));

    let query_list = format!(
        r#"
        query {{
            ordersByUser(userId: "{}", limit: 10) {{
                id
            }}
        }}
        "#,
        Uuid::new_v4()
    );

    let request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(json!({ "query": query_list }).to_string())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let error_msg = json["errors"][0]["message"].as_str().unwrap();

    assert!(error_msg.contains("Connection error"));
}
