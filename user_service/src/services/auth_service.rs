use crate::auth::jwt::generate_jwt;
use crate::dto::user_dto::{AuthResponse, LoginUser as LoginUserDto, NewUser as NewUserDto};
use crate::models::user::{NewUser as DbNewUser, User as DbUser};
use crate::repository::repository;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use jsonwebtoken;
use rand::thread_rng;
use serde::Serialize;
use std::fmt;
use tracing;

#[derive(Debug, Serialize)]
pub enum AuthError {
    DatabaseError(String),
    PasswordHashingError(String),
    InvalidCredentials,
    TokenGenerationError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AuthError::PasswordHashingError(e) => write!(f, "Could not hash password: {}", e),
            AuthError::InvalidCredentials => write!(f, "Invalid phone number or password"),
            AuthError::TokenGenerationError(e) => write!(f, "Could not generate token: {}", e),
        }
    }
}

impl From<String> for AuthError {
    fn from(err: String) -> AuthError {
        AuthError::DatabaseError(err)
    }
}

impl From<DieselError> for AuthError {
    fn from(err: DieselError) -> AuthError {
        match err {
            DieselError::NotFound => {
                AuthError::DatabaseError("The record is not found".to_string())
            }
            DieselError::DatabaseError(kind, info) => {
                if let diesel::result::DatabaseErrorKind::UniqueViolation = kind {
                    return AuthError::DatabaseError(
                        "Email or phone number are already busy.".to_string(),
                    );
                }
                AuthError::DatabaseError(info.message().to_string())
            }
            _ => AuthError::DatabaseError(err.to_string()),
        }
    }
}

impl From<argon2::password_hash::Error> for AuthError {
    fn from(err: argon2::password_hash::Error) -> AuthError {
        AuthError::PasswordHashingError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> AuthError {
        AuthError::TokenGenerationError(err.to_string())
    }
}

pub fn register_user(
    conn: &mut PgConnection,
    new_user_dto: NewUserDto,
) -> Result<DbUser, AuthError> {
    let mut rng = thread_rng();
    let salt = SaltString::generate(&mut rng);
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(new_user_dto.password.as_bytes(), &salt)?
        .to_string();

    let db_new_user = DbNewUser {
        name: new_user_dto.name,
        phone_number: new_user_dto.phone_number,
        email: new_user_dto.email,
        password: hashed_password,
        role: "user".to_string(),
    };

    repository::create(conn, db_new_user)
}

pub fn login_user(
    conn: &mut PgConnection,
    login_user_dto: LoginUserDto,
) -> Result<AuthResponse, AuthError> {
    let user = repository::find_by_phone(conn, &login_user_dto.phone_number)
        .map_err(|_| AuthError::InvalidCredentials)?;

    let parsed_hash = PasswordHash::new(&user.password)?;
    if Argon2::default()
        .verify_password(login_user_dto.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        tracing::warn!("Unsuccessful attempt to enter the user: {}", user.id);
        return Err(AuthError::InvalidCredentials);
    }

    let token = generate_jwt(&user)?;
    tracing::info!("User {} successfully entered the system", user.id);

    Ok(AuthResponse {
        token,
        user_id: user.id,
    })
}
