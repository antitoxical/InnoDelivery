use crate::config::CONFIG;
use crate::models::user::User as DbUser;
use chrono::Utc;
use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

pub fn generate_jwt(user: &DbUser) -> Result<String, JwtError> {
    let expiration = Utc::now() + chrono::Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.timestamp() as usize,
        role: user.role.clone(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
    )
}

pub fn validate_jwt(token: &str) -> Result<Claims, JwtError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(CONFIG.jwt_secret.as_ref()),
        &validation,
    )?;
    Ok(token_data.claims)
}
