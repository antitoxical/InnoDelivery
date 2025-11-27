use crate::helpers;
use httpmock::Method::GET;
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_service_is_user_blocked_error_500() {
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (_app, _pool, server) = helpers::setup_test_app(&db_url).await;
    let user_id = Uuid::new_v4();

    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/internal/users/{}/blocked", user_id));
        then.status(500);
    });

    let result =
        order_service::services::order_service::is_user_blocked(&server.base_url(), user_id).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("User service returned error status: 500"));
}
