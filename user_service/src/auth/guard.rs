use crate::auth::middleware::JwtMiddleware;
use actix_web::{FromRequest, HttpRequest, dev::Payload, error::ErrorForbidden};
use std::future::{Ready, ready};

pub struct UserGuard {
    pub claims: crate::auth::jwt::Claims,
}

impl FromRequest for UserGuard {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let jwt_middleware_future = JwtMiddleware::from_request(req, payload);

        match jwt_middleware_future.into_inner() {
            Ok(jwt_middleware) => {
                if jwt_middleware.claims.role == "user" {
                    ready(Ok(UserGuard {
                        claims: jwt_middleware.claims,
                    }))
                } else {
                    ready(Err(ErrorForbidden(
                        "Insufficient permissions: User role required",
                    )))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

pub struct CourierGuard {
    pub claims: crate::auth::jwt::Claims,
}

impl FromRequest for CourierGuard {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match JwtMiddleware::from_request(req, payload).into_inner() {
            Ok(middleware) => {
                if middleware.claims.role == "courier" {
                    ready(Ok(CourierGuard {
                        claims: middleware.claims,
                    }))
                } else {
                    ready(Err(ErrorForbidden(
                        "Insufficient permissions: Courier role required",
                    )))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}

pub struct AdminGuard {
    pub claims: crate::auth::jwt::Claims,
}

impl FromRequest for AdminGuard {
    type Error = actix_web::Error;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        match JwtMiddleware::from_request(req, payload).into_inner() {
            Ok(middleware) => {
                if middleware.claims.role == "admin" {
                    ready(Ok(AdminGuard {
                        claims: middleware.claims,
                    }))
                } else {
                    ready(Err(ErrorForbidden(
                        "Insufficient permissions: Admin role required",
                    )))
                }
            }
            Err(e) => ready(Err(e)),
        }
    }
}
