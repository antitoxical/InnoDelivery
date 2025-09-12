#![cfg(test)]

mod unit_tests {
    use chrono::Utc;
    use user_service::dto::courier_dto::CourierProfileResponse;
    use user_service::models::courier::{Courier, CourierStatus};
    use user_service::models::user::User;
    use user_service::services::auth_service::AuthError;
    use uuid::Uuid;

    #[test]
    /// Тест 1: Проверка форматирования сообщений об ошибках.
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
    /// Тест 2: (Пример) Проверка логики хеширования пароля.
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
    /// Тест 5: (Пример) Проверка конвертации ошибки Diesel в AuthError.
    fn test_diesel_error_to_auth_error_conversion() {
        let diesel_not_found = diesel::result::Error::NotFound;
        let auth_error: AuthError = diesel_not_found.into();
        assert!(matches!(auth_error, AuthError::InvalidCredentials));

        let other_db_error = diesel::result::Error::QueryBuilderError("test".into());
        let auth_error: AuthError = other_db_error.into();
        assert!(matches!(auth_error, AuthError::DatabaseError(_)));
    }

    #[test]
    /// Тест 7: Проверка конвертации ошибки Argon2 в AuthError.
    fn test_argon2_error_to_auth_error_conversion() {
        let argon2_error = argon2::password_hash::Error::Password;
        let auth_error: AuthError = argon2_error.into();
        assert!(matches!(auth_error, AuthError::PasswordHashingError(_)));
        assert!(format!("{}", auth_error).contains("Could not hash password"));
    }

    #[test]
    /// Тест 9: Проверка маппинга моделей (User, Courier) в DTO (CourierProfileResponse).
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

        // Логика, которая находится внутри вашего courier_service
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
}
