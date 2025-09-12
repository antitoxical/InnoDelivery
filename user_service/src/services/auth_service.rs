use crate::auth::jwt::generate_jwt;
use crate::dto::user_dto::{AuthResponse, LoginUser as LoginUserDto, NewUser as RegisterUserDto};
use crate::models::courier::{CourierStatus, NewCourier};
use crate::models::user::{NewUser as DbNewUser, User as DbUser};
use crate::repository::{courier_repository, user_repository};
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
    ValidationError(String),
    ConnectionError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AuthError::PasswordHashingError(e) => write!(f, "Could not hash password: {}", e),
            AuthError::InvalidCredentials => write!(f, "Invalid phone number or password"),
            AuthError::TokenGenerationError(e) => write!(f, "Could not generate token: {}", e),
            AuthError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AuthError::ConnectionError(e) => write!(f, "Connection error: {}", e),
        }
    }
}

impl From<DieselError> for AuthError {
    fn from(err: DieselError) -> AuthError {
        match err {
            DieselError::NotFound => AuthError::InvalidCredentials,
            DieselError::DatabaseError(kind, info) => {
                if let diesel::result::DatabaseErrorKind::UniqueViolation = kind {
                    return AuthError::ValidationError(
                        "Email or phone number are already in use.".to_string(),
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

pub fn register(
    conn: &mut PgConnection,
    register_data: RegisterUserDto,
) -> Result<DbUser, AuthError> {
    let role_str = match register_data.role.as_str() {
        "user" => "user",
        "courier" => "courier",
        _ => return Err(AuthError::ValidationError("Invalid role specified".into())),
    };

    let salt = SaltString::generate(&mut thread_rng());
    let argon2 = Argon2::default();
    let hashed_password = argon2
        .hash_password(register_data.password.as_bytes(), &salt)?
        .to_string();

    let db_new_user = DbNewUser {
        name: register_data.name,
        phone_number: register_data.phone_number,
        email: register_data.email,
        password: hashed_password,
        role: role_str.to_string(),
    };

    conn.transaction(
        |connection: &mut PgConnection| -> Result<DbUser, diesel::result::Error> {
            let created_user = user_repository::create(connection, &db_new_user)?;

            if role_str == "courier" {
                let new_courier = NewCourier {
                    user_id: created_user.id,
                    status: CourierStatus::Free,
                };
                courier_repository::create(connection, &new_courier)?;
            }

            Ok(created_user)
        },
    )
    .map_err(AuthError::from)
}

pub fn login(conn: &mut PgConnection, login_data: LoginUserDto) -> Result<AuthResponse, AuthError> {
    let user = user_repository::find_by_phone(conn, &login_data.phone_number)?;

    let parsed_hash = PasswordHash::new(&user.password)?;
    if Argon2::default()
        .verify_password(login_data.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        tracing::warn!(
            "Unsuccessful login attempt for phone: {}",
            login_data.phone_number
        );
        return Err(AuthError::InvalidCredentials);
    }

    let token = generate_jwt(&user)?;
    tracing::info!("User {} successfully logged in", user.id);

    Ok(AuthResponse {
        token,
        user_id: user.id,
    })
}
