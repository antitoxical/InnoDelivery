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
    use chrono::Utc;
    use jsonwebtoken::{DecodingKey, Validation, decode};
    use user_service::auth::jwt::{Claims, generate_jwt, validate_jwt};
    use user_service::dto::courier_dto::CourierProfileResponse;
    use user_service::dto::user_dto::UpdateUser;
    use user_service::models::courier::{Courier, CourierStatus};
    use user_service::models::user::User;
    use user_service::services::auth_service::AuthError;
    use uuid::Uuid;

    fn create_test_user() -> User {
        let now = Utc::now().naive_utc();
        User {
            id: Uuid::new_v4(),
            name: "Test User".to_string(),
            phone_number: "1234567890".to_string(),
            email: "test@example.com".to_string(),
            password: "hashed_password".to_string(),
            role: "user".to_string(),
            favorite_address: None,
            is_blocked: false,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    /// Test 1: Checking the formatting of error messages.
    fn test_auth_error_display_formats() {
        let db_error = AuthError::DatabaseError("Connection failed".to_string());
        let validation_error = AuthError::ValidationError("Email is invalid".to_string());
        assert_eq!(format!("{}", db_error), "Database error: Connection failed");
        assert_eq!(
            format!("{}", validation_error),
            "Validation error: Email is invalid"
        );
    }

    #[test]
    /// Test 2: Checking the logic of hashing password.
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
        assert!(
            argon2
                .verify_password(b"wrong_password", &parsed_hash)
                .is_err()
        );
    }

    #[test]
    /// Test 3: Checking the conversion of the Diesel Error in AuthError.
    fn test_diesel_error_to_auth_error_conversion() {
        let diesel_not_found = diesel::result::Error::NotFound;
        let auth_error: AuthError = diesel_not_found.into();
        assert!(matches!(auth_error, AuthError::InvalidCredentials));

        let other_db_error = diesel::result::Error::QueryBuilderError("test".into());
        let auth_error: AuthError = other_db_error.into();
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));
    }

    #[test]
    /// Тest 4: Checking JWT generation and contents of its Claims.
    fn test_jwt_generation_and_claims() {
        dotenv::dotenv().ok();
        let mut user = create_test_user();
        user.role = "user".to_string();

        let token = generate_jwt(&user).expect("Failed to generate JWT");
        let decoded = validate_jwt(&token).expect("Failed to validate token");
        assert_eq!(decoded.sub, user.id.to_string());
        assert_eq!(decoded.role, user.role);
    }

    #[test]
    /// Test 5: Checking the conversion of the Argon2 error in Autherror.
    fn test_argon2_error_to_auth_error_conversion() {
        let argon2_error = argon2::password_hash::Error::Password;
        let auth_error: AuthError = argon2_error.into();
        assert!(matches!(auth_error, AuthError::PasswordHashingError(_)));
        assert!(format!("{}", auth_error).contains("Could not hash password"));
    }

    #[test]
    /// Test 6: Checking the conversion of the JWT error in AuthError.
    fn test_jwt_error_to_auth_error_conversion() {
        let jwt_error =
            jsonwebtoken::errors::Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken);
        let auth_error: AuthError = jwt_error.into();
        assert!(matches!(auth_error, AuthError::TokenGenerationError(_)));
    }

    #[test]
    /// Test 7: Checking the Mapping Models (User, Courier) in DTO (CourierProfileResponse).
    fn test_model_to_dto_mapping_for_courier_profile() {
        let now = Utc::now().naive_utc();
        let user_id = Uuid::new_v4();
        let user = User {
            id: user_id,
            name: "John Doe".to_string(),
            phone_number: "+12345".to_string(),
            email: "john@doe.com".to_string(),
            password: "hash".to_string(),
            role: "courier".to_string(),
            favorite_address: None,
            is_blocked: false,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        };
        let courier = Courier {
            id: Uuid::new_v4(),
            user_id,
            status: CourierStatus::Busy,
            rating: 4.8,
            created_at: now,
            updated_at: now,
        };

        let dto = CourierProfileResponse {
            id: user.id,
            name: user.name,
            phone_number: user.phone_number,
            email: user.email,
            status: courier.status,
            rating: courier.rating,
        };

        assert_eq!(dto.id, user.id);
        assert_eq!(dto.name, "John Doe");
        assert_eq!(dto.status, CourierStatus::Busy);
        assert_eq!(dto.rating, 4.8);
    }

    #[test]
    /// Test 8: Checking that the UniqueViolation error from Diesel is correctly converted.
    fn test_diesel_unique_violation_error_conversion() {
        let unique_violation_error = diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            Box::new("duplicate key value violates unique constraint".to_string()),
        );
        let auth_error: AuthError = unique_violation_error.into();
        assert!(matches!(auth_error, AuthError::ValidationError(_)));
        assert!(format!("{}", auth_error).contains("Email or phone number are already in use."));
    }

    #[test]
    /// Test 9 : Checking that the validation of JWT fails for token with a wrong signature.
    fn test_jwt_validation_fails_for_invalid_signature() {
        dotenv::dotenv().ok();
        let mut user = create_test_user();
        user.role = "user".to_string();
        let token = generate_jwt(&user).unwrap();

        let wrong_secret = "a_completely_different_secret";
        let decoding_key = DecodingKey::from_secret(wrong_secret.as_ref());
        let validation = Validation::default();

        let result = decode::<Claims>(&token, &decoding_key, &validation);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().kind(),
            jsonwebtoken::errors::ErrorKind::InvalidSignature
        ));
    }

    #[test]
    /// Test 10: Checking the conversion of the Diesel ConnectionError in AuthError.
    fn test_diesel_connection_error_conversion() {
        let conn_error = diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::NotNullViolation,
            Box::new("connection refused".to_string()),
        );
        let auth_error: AuthError = conn_error.into();
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));
        assert!(format!("{}", auth_error).contains("connection refused"));
    }

    #[test]
    /// Test 11:Check DTO creation for updating user.
    fn test_update_user_dto_creation() {
        let update_data = UpdateUser {
            name: Some("New Name".to_string()),
            email: None,
            phone_number: Some("12345".to_string()),
            favorite_address: None,
        };
        assert_eq!(update_data.name.unwrap(), "New Name");
        assert!(update_data.email.is_none());
    }
}
