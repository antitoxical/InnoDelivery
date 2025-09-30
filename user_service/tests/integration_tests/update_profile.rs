use actix_web::{App, http::StatusCode, test, web};
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use serde_json::json;
use user_service::{
    config::{config_auth, config_courier, config_user},
    db::{DbPool, create_db_pool},
    dto::user_dto::AuthResponse,
    models::user::User as DbUser,
};

use crate::fakers;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");

#[actix_web::test]
async fn test_update_user_profile() {
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
            .app_data(web::Data::new(pool.clone()))
            .configure(config_auth)
            .service(web::scope("/api").configure(config_user))
            .service(web::scope("/courier").configure(config_courier)),
    )
    .await;

    let phone = fakers::fake_phone();
    let email = fakers::fake_email();

    let req = test::TestRequest::post()
        .uri("/auth/register")
        .set_json(&json!({
            "name": "Test User", "phone_number": &phone, "email": &email,
            "password": "password123", "role": "user"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(&json!({ "phone_number": &phone, "password": "password123" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let auth: AuthResponse = test::read_body_json(resp).await;
    println!("user_id: {:?}, token: {}", auth.user_id, auth.token);
    let token = auth.token;

    let req = test::TestRequest::get()
        .uri("/api/api/profile")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    println!("profile after login: status = {:?}", resp.status());
    if resp.status() == StatusCode::OK {
        let profile: DbUser = test::read_body_json(resp).await;
        println!("profile after login: name = {}", profile.name);
    }

    let new_name = "Updated Name";
    let req = test::TestRequest::patch()
        .uri("/api/api/update")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({ "name": new_name }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: DbUser = test::read_body_json(resp).await;
    assert_eq!(updated.name, new_name);
}
