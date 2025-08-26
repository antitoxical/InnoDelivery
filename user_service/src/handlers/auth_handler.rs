
use actix_web::{web, HttpResponse, Responder};
use crate::dto::user_dto::{NewUser, LoginUser};
use crate::services::auth_service::{self, AuthError};
use crate::db;
use crate::db::DbPool;

pub async fn register_user(
    pool: web::Data<DbPool>,
    new_user_dto: web::Json<NewUser>,
) -> impl Responder {
    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        auth_service::register_user(&mut conn, new_user_dto.into_inner())
    })
        .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Created().json(user),
        Ok(Err(e)) => match e {
            AuthError::DatabaseError(msg) => {
                if msg.contains("duplicate key value") {
                    HttpResponse::Conflict().body("User with such a phone number or email already exists.")
                } else {
                    HttpResponse::InternalServerError().body(msg)
                }
            },
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error."),
    }
}

pub async fn login_user(
    pool: web::Data<DbPool>,
    login_user_dto: web::Json<LoginUser>,
) -> impl Responder {
    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        auth_service::login_user(&mut conn, login_user_dto.into_inner())
    })
        .await;

    match result {
        Ok(Ok(response)) => HttpResponse::Ok().json(response),
        Ok(Err(e)) => match e {
            AuthError::InvalidCredentials => HttpResponse::Unauthorized().body(e.to_string()),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error."),
    }
}
