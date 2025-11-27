use crate::helpers;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_graphql_playground_endpoint() {
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (app, _pool, _server) = helpers::setup_test_app(&db_url).await;

    let request = Request::get("/").body(axum::body::Body::empty()).unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body_bytes);

    assert!(body_str.contains("GraphiQL"));
}
