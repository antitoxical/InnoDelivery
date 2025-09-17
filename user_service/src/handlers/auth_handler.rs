use crate::db;
use crate::db::DbPool;
use crate::dto::user_dto::{LoginUser, NewUser as RegisterUserDto};
use crate::services::auth_service::{self, AuthError};
use actix_web::{HttpResponse, Responder, web};

pub async fn register_user(
    pool: web::Data<DbPool>,
    new_user_dto: web::Json<RegisterUserDto>,
) -> impl Responder {
    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        auth_service::register(&mut conn, new_user_dto.into_inner())
    })
    .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Created().json(user),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            AuthError::ValidationError(msg) => {
                if msg.contains("already in use") {
                    HttpResponse::Conflict().body(msg)
                } else {
                    HttpResponse::BadRequest().body(msg)
                }
            },
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn login_user(
    pool: web::Data<DbPool>,
    login_user_dto: web::Json<LoginUser>,
) -> impl Responder {
    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        auth_service::login(&mut conn, login_user_dto.into_inner())
    })
    .await;

    match result {
        Ok(Ok(response)) => HttpResponse::Ok().json(response),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            AuthError::InvalidCredentials => HttpResponse::Unauthorized().body(e.to_string()),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
