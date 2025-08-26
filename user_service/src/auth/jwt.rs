
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use chrono::Utc;
use std::env;
use crate::models::user::User as DbUser;
use jsonwebtoken::errors::Error as JwtError;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn generate_jwt(user: &DbUser) -> Result<String, JwtError> {
    let now = Utc::now();
    let expiration = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.timestamp() as usize,
    };
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
}

pub fn validate_jwt(token: &str) -> Result<Claims, JwtError> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation)?;

    Ok(token_data.claims)
}
