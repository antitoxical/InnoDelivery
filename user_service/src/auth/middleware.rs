use crate::auth::jwt::{Claims, validate_jwt};
use actix_web::{FromRequest, HttpRequest, dev::Payload, error::ErrorUnauthorized, http};
use std::future::{Ready, ready};
use tracing;

pub struct JwtMiddleware {
    pub claims: Claims,
}

impl FromRequest for JwtMiddleware {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let auth_header = match req.headers().get(http::header::AUTHORIZATION) {
            Some(header) => header,
            None => {
                return ready(Err(ErrorUnauthorized("There is no authentication token")));
            }
        };

        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(ErrorUnauthorized(
                    "Incorrect format of authentication headline",
                )));
            }
        };

        if !auth_str.starts_with("Bearer ") {
            return ready(Err(ErrorUnauthorized(
                "The wrong authentication scheme is expected 'Bearer token'",
            )));
        }

        let token = &auth_str["Bearer ".len()..];

        match validate_jwt(token) {
            Ok(claims) => ready(Ok(JwtMiddleware { claims })),
            Err(e) => {
                tracing::warn!("JWT validation error: {}", e);
                ready(Err(ErrorUnauthorized("Unimportant or expired token")))
            }
        }
    }
}
