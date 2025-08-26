use diesel::prelude::*;
use crate::models::user::{NewUser as DbNewUser, User as DbUser};
use crate::schema::users;
use crate::dto::user_dto::{NewUser as NewUserDto, LoginUser as LoginUserDto, AuthResponse};
use crate::auth::jwt::generate_jwt;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rand::thread_rng;
use diesel::result::Error as DieselError;
use std::fmt;
use jsonwebtoken;
use serde::Serialize;
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
        if let DieselError::NotFound = err {
            return AuthError::DatabaseError("The user was not found".to_string());
        }
        AuthError::DatabaseError(err.to_string())
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


pub fn register_user(conn: &mut PgConnection, new_user_dto: NewUserDto) -> Result<DbUser, AuthError> {
    let mut rng = thread_rng();
    let salt = SaltString::generate(&mut rng);
    let argon2 = Argon2::default();

    let hashed_password = argon2.hash_password(new_user_dto.password.as_bytes(), &salt)?
        .to_string();

    let db_new_user = DbNewUser {
        name: new_user_dto.name,
        phone_number: new_user_dto.phone_number,
        email: new_user_dto.email,
        password: hashed_password,
        role: "user".to_string(),
    };

    let new_db_user = diesel::insert_into(users::table)
        .values(&db_new_user)
        .returning(DbUser::as_returning())
        .get_result::<DbUser>(conn)?;

    Ok(new_db_user)
}

pub fn login_user(conn: &mut PgConnection, login_user_dto: LoginUserDto) -> Result<AuthResponse, AuthError> {
    let user = users::table
        .filter(users::phone_number.eq(&login_user_dto.phone_number))
        .select(DbUser::as_select())
        .first::<DbUser>(conn)
        .map_err(|e| {
            tracing::error!("Database error when searching for a user'{}': {:?}", &login_user_dto.phone_number, e);
            if let DieselError::NotFound = e {
                return AuthError::InvalidCredentials;
            }
            AuthError::from(e)
        })?;

    let parsed_hash = PasswordHash::new(&user.password)
        .map_err(|e| {
            tracing::error!("Failed to parse the hash password from the database for the user {}: {:?}", user.id, e);
            e
        })?;

    if Argon2::default().verify_password(login_user_dto.password.as_bytes(), &parsed_hash).is_err() {
        tracing::warn!("Unsuccessful attempt to enter the user: {}", user.id);
        return Err(AuthError::InvalidCredentials);
    }

    let token = generate_jwt(&user)
        .map_err(|e| {
            tracing::error!("JWT generation error for user {}: {:?}", user.id, e);
            e
        })?;

    tracing::info!("User {} successfully entered the system", user.id);

    Ok(AuthResponse {
        token,
        user_id: user.id,
    })
}
