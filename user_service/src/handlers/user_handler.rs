use actix_web::{web, HttpResponse, Responder};
use crate::auth::middleware::JwtMiddleware;
use crate::db;
use crate::db::DbPool;
use crate::services::user_service;
use crate::dto::user_dto::UpdateUser;
use crate::services::auth_service::AuthError;
use uuid::Uuid;


pub async fn get_profile(
    pool: web::Data<DbPool>,
    auth: JwtMiddleware,
) -> impl Responder {
    let user_id_str = &auth.claims.sub;

    let user_id = match Uuid::parse_str(user_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("The wrong format of the user ID in the token"),
    };

    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        user_service::get_user_profile(&mut conn, user_id)
    }).await;

    match result {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(service_error)) => match service_error {
            AuthError::DatabaseError(msg) if msg == "The record is not found" => {
                HttpResponse::NotFound().body(msg)
            },
            _ => HttpResponse::InternalServerError().body(service_error.to_string()),
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error."),
    }
}


pub async fn update_profile(
    pool: web::Data<DbPool>,
    auth: JwtMiddleware,
    update_data: web::Json<UpdateUser>,
) -> impl Responder {
    let user_id_str = &auth.claims.sub;

    let user_id = match Uuid::parse_str(user_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("The wrong format of the user ID in the token"),
    };

    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        user_service::update_user_profile(&mut conn, user_id, update_data.into_inner())
    }).await;

    match result {
        Ok(Ok(updated_user)) => HttpResponse::Ok().json(updated_user),
        Ok(Err(service_error)) => match service_error {
            AuthError::DatabaseError(msg) if msg.contains("Already busy") => {
                HttpResponse::Conflict().body(msg)
            },
            AuthError::DatabaseError(msg) if msg == "The record is not found" => {
                HttpResponse::NotFound().body(msg)
            },
            _ => HttpResponse::InternalServerError().body(service_error.to_string()),
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

pub async fn delete_profile(
    pool: web::Data<DbPool>,
    auth: JwtMiddleware,
) -> impl Responder {
    let user_id_str = &auth.claims.sub;

    let user_id = match Uuid::parse_str(user_id_str) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("The wrong format of the user ID in the token"),
    };

    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        user_service::soft_delete_user(&mut conn, user_id)
    }).await;

    match result {
        Ok(Ok(num_deleted)) if num_deleted > 0 => HttpResponse::NoContent().finish(),
        Ok(Ok(_)) => HttpResponse::NotFound().body("The user was not found."),
        Ok(Err(service_error)) => match service_error {
            AuthError::DatabaseError(msg) if msg == "The record is not found" => {
                HttpResponse::NotFound().body(msg)
            },
            _ => HttpResponse::InternalServerError().body(service_error.to_string()),
        },
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}

pub async fn get_users(pool: web::Data<DbPool>) -> impl Responder {
    let result = web::block(move || {
        let mut conn = db::get_conn_from_pool(&pool)?;
        user_service::get_all_users(&mut conn)
    }).await;

    match result {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e.to_string()),
        Err(_) => HttpResponse::InternalServerError().body("Internal Server Error"),
    }
}
