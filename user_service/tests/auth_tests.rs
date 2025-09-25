use chrono::Utc;
use jsonwebtoken::{DecodingKey, Validation, decode};
use user_service::auth::jwt::{Claims, generate_jwt, validate_jwt};
use user_service::models::user::User;
use uuid::Uuid;

fn create_test_user(role: &str) -> User {
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
    assert!(
        argon2
            .verify_password(b"wrong_password", &parsed_hash)
            .is_err()
    );
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
    assert!(matches!(
        result.unwrap_err().kind(),
        jsonwebtoken::errors::ErrorKind::InvalidSignature
    ));
}
