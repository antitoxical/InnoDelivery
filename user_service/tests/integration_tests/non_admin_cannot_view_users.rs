use actix_web::{App, http::StatusCode, test, web};
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use serde_json::json;
use user_service::{
    admin::init_admin,
    config::{config_admin, config_auth, config_courier, config_user},
    db::{DbPool, create_db_pool},
};

use crate::fakers;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");

#[actix_web::test]
async fn test_non_admin_cannot_view_users() {
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

    init_admin(&pool).await.ok();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(config_auth)
            .configure(config_user)
            .configure(config_courier)
            .configure(config_admin),
    )
    .await;

    let phone = fakers::fake_phone();
    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&json!({
            "name": "Regular User",
            "phone_number": &phone,
            "email": fakers::fake_email(),
            "password": "user123",
            "role": "user"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&json!({ "phone_number": &phone, "password": "user123" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let auth: serde_json::Value = test::read_body_json(resp).await;
    let user_token = auth["token"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri("/users_admin/all")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
