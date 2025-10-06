use user_service::services::auth_service::AuthError;

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
