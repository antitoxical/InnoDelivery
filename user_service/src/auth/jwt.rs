use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use chrono::Utc;
use crate::models::user::User as DbUser;
use jsonwebtoken::errors::Error as JwtError;
use serde::{Deserialize, Serialize};
use crate::config;
use crate::config::JWT_SECRET;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_jwt(user: &DbUser) -> Result<String, JwtError> {
    let expiration = Utc::now() + chrono::Duration::hours(24);
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.timestamp() as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET.as_ref()))
}

pub fn validate_jwt(token: &str) -> Result<Claims, JwtError> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET.as_ref()), &validation)?;
    Ok(token_data.claims)
}
