use actix_web::{App, http::StatusCode, test, web};
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use serde_json::json;
use user_service::{
    config::{config_auth, config_courier, config_user},
    db::{DbPool, create_db_pool},
};

use crate::fakers;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");

#[actix_web::test]
async fn test_registration_fails_on_duplicate_phone() {
    dotenv::dotenv().ok();

    let pool: DbPool = create_db_pool().expect("Failed to create test DB pool");
    {
        let mut conn = pool.get().expect("Failed to get connection for cleaning");
        conn.run_pending_migrations(MIGRATIONS).ok();
        diesel::sql_query("TRUNCATE TABLE couriers CASCADE")
            .execute(&mut conn)
            .ok();
        diesel::sql_query("TRUNCATE TABLE users CASCADE")
            .execute(&mut conn)
            .ok();
    }

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool))
            .configure(config_auth)
            .service(web::scope("/api").configure(config_user))
            .service(web::scope("/courier").configure(config_courier)),
    )
    .await;

    let phone = fakers::fake_phone();
    let email1 = fakers::fake_email();
    let email2 = fakers::fake_email();

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&json!({
            "name": "First User", "phone_number": &phone, "email": email1,
            "password": "password123", "role": "user"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&json!({
            "name": "Second User", "phone_number": &phone, "email": email2,
            "password": "password123", "role": "user"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}
