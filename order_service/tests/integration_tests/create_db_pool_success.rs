use order_service::db;
use std::env;

#[tokio::test]
async fn test_create_db_pool_success() {
    dotenvy::dotenv().ok();
    let test_db_url =
        env::var("DATABASE_URL_ORDER_TEST").expect("DATABASE_URL_ORDER_TEST must be set");

    env::set_var("DATABASE_URL_ORDER", &test_db_url);

    let pool_result = db::create_db_pool();

    assert!(
        pool_result.is_ok(),
        "Failed to create DB pool with valid config"
    );
    let _pool = pool_result.unwrap();

    env::remove_var("DATABASE_URL_ORDER");
}
