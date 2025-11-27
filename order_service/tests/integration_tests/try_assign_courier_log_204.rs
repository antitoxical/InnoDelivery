use crate::helpers;
use httpmock::Method::POST;
use std::env;

#[tokio::test]
async fn test_service_try_assign_courier_204_log() {
    let db_url = env::var("DATABASE_URL_ORDER_TEST").expect("Env var not set");
    let (_app, _pool, server) = helpers::setup_test_app(&db_url).await;

    server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(204);
    });

    let result =
        order_service::services::order_service::try_assign_courier(&server.base_url()).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
