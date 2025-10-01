use actix_web::{App, http::StatusCode, test, web};
use chrono::{Duration, Utc};
use diesel::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::json;
use user_service::{
    auth::jwt::Claims,
    config::{config_auth, config_courier, config_user},
    db::{DbPool, create_db_pool},
    dto::user_dto::AuthResponse,
};

use crate::fakers;

pub const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("migrations");

#[actix_web::test]
async fn test_access_with_expired_token() {
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
            .configure(config_user)
            .configure(config_courier),
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
    let auth: AuthResponse = test::read_body_json(resp).await;
    let user_id = auth.user_id;

    let claims = Claims {
        sub: user_id.to_string(),
        role: "user".to_string(),
        exp: (Utc::now() - Duration::hours(1)).timestamp() as usize,
    };
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expired_token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to generate expired JWT");

    let req = test::TestRequest::get()
        .uri("/users/profile")
        .insert_header(("Authorization", format!("Bearer {}", expired_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
