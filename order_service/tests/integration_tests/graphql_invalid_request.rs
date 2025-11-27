use crate::helpers;
use axum::http::{Request, StatusCode};
use dotenvy::dotenv;
use std::env;
use tower::ServiceExt;

#[tokio::test]
async fn graphql_error_async_graphql_error() {
    dotenv().ok();
    let db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST needs to be set");
    let (app, _pool, _server) = helpers::setup_test_app(&*db_url).await;

    let mutation = r#"
        mutation {
            rateOrder(id: "invalid-uuid", rating: 5.0, userId: "invalid-uuid", ratingWindowMinutes: 1440) {
                id
                rating
            }
        }
    "#;

    let request = Request::post("/")
        .header("Content-Type", "application/json")
        .body(mutation.to_string())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
