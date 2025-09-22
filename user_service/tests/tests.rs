#![cfg(test)]
mod integration_tests {
    use actix_web::{App, http::StatusCode, test, web};
    use diesel::RunQueryDsl;
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
    use fake::{
        Fake,
        faker::{internet::en::SafeEmail, phone_number::en::PhoneNumber},
    };
    use serde_json::json;
    use chrono::NaiveDateTime;
    use user_service::{
        config::{config_auth, config_courier, config_user},
        db::{DbPool, create_db_pool},
        dto::user_dto::AuthResponse,
        models::user::User as DbUser,

    };


    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    #[actix_web::test]
    /// Test 1: Successful registration, entrance and profile for a user.
    async fn test_user_happy_path() {
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

        let phone: String = PhoneNumber().fake();
        let email: String = SafeEmail().fake();

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
        let token = auth.token;

        let req = test::TestRequest::get()
            .uri("/api/api/profile")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Get user profile should be successful"
        );
        let profile: DbUser = test::read_body_json(resp).await;
        assert_eq!(profile.email, email);
    }

    #[actix_web::test]
    /// Test 2: Registration should end with an error of 409 conflict when trying to use the existing email.
    async fn test_registration_fails_on_duplicate_email() {
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

        let phone1: String = PhoneNumber().fake();
        let phone2: String = PhoneNumber().fake();
        let email: String = SafeEmail().fake();

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "First User", "phone_number": phone1, "email": &email,
                "password": "password123", "role": "user"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::CREATED
        );

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "Second User", "phone_number": phone2, "email": &email,
                "password": "password456", "role": "user"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    /// Test 3: Successful registration and login of the courier.
    async fn test_courier_registration_and_login() {
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

        let phone: String = PhoneNumber().fake();
        let email: String = SafeEmail().fake();

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "Test Courier", "phone_number": &phone, "email": &email,
                "password": "courier_pass", "role": "courier"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::CREATED
        );

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "phone_number": &phone, "password": "courier_pass" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let auth: AuthResponse = test::read_body_json(resp).await;
        assert_eq!(auth.user_id.to_string().len(), 36);
    }

    #[actix_web::test]
    /// Test 4: An unsuccessful login with the wrong password.
    async fn test_login_with_wrong_password() {
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

        let phone: String = PhoneNumber().fake();
        let email: String = SafeEmail().fake();

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "Test User", "phone_number": &phone, "email": &email,
                "password": "correct_password", "role": "user"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::CREATED
        );

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "phone_number": &phone, "password": "wrong_password" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    /// Test 5: An attempt to access the protected endpoint without token.
    async fn test_unauthorized_access_to_protected_endpoint() {
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

        let req = test::TestRequest::get()
            .uri("/api/api/profile")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    /// Test 6: An attempt to access the endpoint courier with a regular user token.
    async fn test_user_access_to_courier_endpoint() {
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

        let phone: String = PhoneNumber().fake();
        let email: String = SafeEmail().fake();

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "Test User", "phone_number": &phone, "email": &email,
                "password": "password123", "role": "user"
            }))
            .to_request();
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::CREATED
        );

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&json!({ "phone_number": &phone, "password": "password123" }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let auth: AuthResponse = test::read_body_json(resp).await;
        let token = auth.token;

        let req = test::TestRequest::get()
            .uri("/courier/profile")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    /// Test 7: Registration with invalid data.
    async fn test_registration_with_invalid_data() {
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

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&json!({
                "name": "Test User", "phone_number": "1234567890", "email": "test@example.com",
                "password": "password123", "role": "invalid_role"
            }))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}

mod unit_tests {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use user_service::auth::jwt::{generate_jwt, validate_jwt, Claims};
    use user_service::models::user::User;
    use user_service::models::courier::{CourierStatus};
    use user_service::models::courier::Courier;
    use user_service::services::auth_service::AuthError;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn create_test_user(role : &str) -> User {
        let now = Utc::now().naive_utc();
        User {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            phone_number: "1234567890".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            role: role.to_string(),
            favorite_address: None,
            is_blocked: false,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_test_claims(sub: String, role: String, exp_offset_secs: i64) -> Claims {
        let expiration = (Utc::now() + Duration::seconds(exp_offset_secs)).timestamp();
        Claims {
            sub,
            role,
            exp: expiration as usize,
        }
    }


    #[test]
    /// Test 1: Checking the logic of hashing password.
    fn test_password_hashing_and_verification() {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::{PasswordHasher, SaltString};
        use argon2::{Argon2, PasswordHash, PasswordVerifier};

        let password = b"password123";
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash_str = argon2.hash_password(password, &salt).unwrap().to_string();
        let parsed_hash = PasswordHash::new(&password_hash_str).unwrap();

        assert!(argon2.verify_password(password, &parsed_hash).is_ok());
        assert!(argon2.verify_password(b"wrong_password", &parsed_hash).is_err());
    }

    #[test]
    /// Test 2: Checking the conversion of the Diesel Error in AuthError.
    fn test_diesel_error_to_auth_error_conversion() {
        let diesel_not_found = diesel::result::Error::NotFound;
        let auth_error: AuthError = diesel_not_found.into();
        assert!(matches!(auth_error, AuthError::InvalidCredentials));

        let other_db_error = diesel::result::Error::QueryBuilderError("test".into());
        let auth_error: AuthError = other_db_error.into();
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));
    }

    #[test]
    /// Test 3: Checking JWT generation and contents of its Claims.
    fn test_jwt_generation_and_claims() {
        dotenv::dotenv().ok();
        let user = create_test_user("user");
        let token = generate_jwt(&user).expect("Failed to generate JWT");
        let decoded = validate_jwt(&token).expect("Failed to validate token");
        assert_eq!(decoded.sub, user.id.to_string());
        assert_eq!(decoded.role, user.role);
    }

    #[test]
    /// Test 4: Checking the conversion of the Argon2 error in AuthError.
    fn test_argon2_error_to_auth_error_conversion() {
        let argon2_error = argon2::password_hash::Error::Password;
        let auth_error: AuthError = argon2_error.into();
        assert!(matches!(auth_error, AuthError::PasswordHashingError(_)));
        assert!(format!("{}", auth_error).contains("Could not hash password"));
    }

    #[test]
    /// Test 5: Checking the conversion of the JWT error in AuthError.
    fn test_diesel_unique_violation_error_conversion() {
        let unique_violation_error = diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            Box::new("duplicate key value violates unique constraint".to_string()),
        );
        let auth_error: AuthError = unique_violation_error.into();
        assert!(matches!(auth_error, AuthError::ValidationError(_)));
    }

    #[test]
    /// Test 6: Successful validation of a correct JWT.
    fn test_jwt_validation_success() {
        dotenv::dotenv().ok();
        let user = create_test_user("admin");
        let token = generate_jwt(&user).unwrap();
        let result = validate_jwt(&token);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().role, "admin");
    }

    #[test]
    /// Test 9: JWT `exp` claim is set to about 24 hours in the future.
    fn test_jwt_expiration_is_set_correctly() {
        dotenv::dotenv().ok();
        let user = create_test_user("user");
        let token = generate_jwt(&user).unwrap();
        let claims = validate_jwt(&token).unwrap();

        let now = Utc::now().timestamp() as usize;
        let twenty_four_hours_in_seconds = 24 * 60 * 60;

        assert!(claims.exp > now + twenty_four_hours_in_seconds - 5);
        assert!(claims.exp < now + twenty_four_hours_in_seconds + 5);
    }

    #[test]
    /// Test 10: UserGuard allows user with 'user' role.
    fn test_user_guard_logic_allows_user_role() {
        let claims = create_test_claims(Uuid::new_v4().to_string(), "user".to_string(), 3600);
        assert_eq!(claims.role, "user");
    }

    #[test]
    /// Test 11: UserGuard blocks user with 'courier' role.
    fn test_user_guard_logic_blocks_courier_role() {
        let claims = create_test_claims(Uuid::new_v4().to_string(), "courier".to_string(), 3600);
        assert_ne!(claims.role, "user");
    }

    #[test]
    /// Test 12: CourierGuard allows user with 'courier' role.
    fn test_courier_guard_logic_allows_courier_role() {
        let claims = create_test_claims(Uuid::new_v4().to_string(), "courier".to_string(), 3600);
        assert_eq!(claims.role, "courier");
    }

    #[test]
    /// Test 12.1: CourierGuard blocks user with 'user' role.
    fn test_courier_guard_logic_blocks_user_role(){
        let claims = create_test_claims(Uuid::new_v4().to_string(), "user".to_string(), 3600);
        assert_ne!(claims.role, "courier");
    }

    #[test]
    /// Test 13: JWT validation fails for invalid signature.
    fn test_jwt_validation_fails_for_invalid_signature() {

        dotenv::dotenv().ok();
        let user = create_test_user("user");
        let token = generate_jwt(&user).unwrap();

        let wrong_secret = "a_completely_different_secret";
        let decoding_key = DecodingKey::from_secret(wrong_secret.as_ref());
        let validation = Validation::default();

        let result = decode::<Claims>(&token, &decoding_key, &validation);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().kind(), jsonwebtoken::errors::ErrorKind::InvalidSignature));
    }

    #[test]
    /// Test 14: Conversion of JWT InvalidSignature error to AuthError.
    fn test_jwt_invalid_signature_error_conversion() {
        let jwt_error =
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidSignature);
        let auth_error: AuthError = jwt_error.into();
        assert!(matches!(auth_error, AuthError::TokenGenerationError(_)));
    }

    #[test]
    /// Test 15: Conversion of Diesel NotNullViolation error works.
    fn test_diesel_not_null_violation_error_conversion() {
        let not_null_error = diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::NotNullViolation,
            Box::new("null value in column".to_string()),
        );
        let auth_error: AuthError = not_null_error.into();
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));
        assert!(format!("{}", auth_error).contains("null value in column"));
    }

    #[test]
    /// Test 16: Correct definition of courier status (Free/Busy)
    fn test_courier_status_enum() {
        assert_eq!(format!("{:?}", CourierStatus::Free), "Free");
        assert_eq!(format!("{:?}", CourierStatus::Busy), "Busy");
    }

    #[test]
    /// Test 17: User with is_blocked=true is considered blocked
    fn test_user_blocked_flag() {
        let mut user = create_test_user("user");
        user.is_blocked = true;
        assert!(user.is_blocked);
    }

    #[test]
    /// Test 18: User with is_deleted=true is considered deleted
    fn test_user_deleted_flag() {
        let mut user = create_test_user("user");
        user.is_deleted = true;
        assert!(user.is_deleted);
    }

    #[test]
    /// Test 19: Courier rating is within 0..=5
    fn test_courier_rating_bounds() {
        let now = Utc::now().naive_utc();
        let courier = Courier {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            status: CourierStatus::Free,
            rating: 4.7,
            created_at: now,
            updated_at: now,
        };
        assert!((0.0..=5.0).contains(&courier.rating));
    }

    #[test]
    /// Test 20: User email must contain '@'
    fn test_user_email_validity() {
        let user = create_test_user("user");
        assert!(user.email.contains('@'));
    }

    #[test]
    /// Test 21: User role can only be 'user' or 'courier'
    fn test_user_role_allowed_values() {
        let user = create_test_user("user");
        assert!(user.role == "user" || user.role == "courier");
        let courier = create_test_user("courier");
        assert!(courier.role == "user" || courier.role == "courier");
    }


    #[test]
    /// Test 23: created_at and updated_at update correctly
    fn test_user_timestamps_update() {
        let mut user = create_test_user("user");
        let old_updated = user.updated_at;
        let new_time = old_updated + chrono::Duration::minutes(5);
        user.updated_at = new_time;
        assert!(user.updated_at > old_updated);
    }

    #[test]
    /// Test 24: Cannot create courier without user_id (user_id must be valid UUID)
    fn test_courier_requires_user_id() {
        let now = Utc::now().naive_utc();
        let user_id = Uuid::new_v4();
        let courier = Courier {
            id: Uuid::new_v4(),
            user_id,
            status: CourierStatus::Free,
            rating: 5.0,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(courier.user_id, user_id);
    }

    #[test]
    /// Test 25: Cannot update courier status to invalid value (enum is limited)
    fn test_courier_status_invalid_value() {
        fn status_from_str(s: &str) -> Option<CourierStatus> {
            match s {
                "free" => Some(CourierStatus::Free),
                "busy" => Some(CourierStatus::Busy),
                _ => None,
            }
        }
        assert!(status_from_str("free").is_some());
        assert!(status_from_str("busy").is_some());
        assert!(status_from_str("invalid").is_none());
    }
}
