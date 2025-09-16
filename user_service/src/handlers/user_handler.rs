use crate::auth::guard::UserGuard;
use crate::db;
use crate::db::DbPool;
use crate::dto::user_dto::UpdateUser;
use crate::services::auth_service::AuthError;
use crate::services::user_service;
use actix_web::{HttpResponse, Responder, web};
use uuid::Uuid;

pub async fn get_profile(pool: web::Data<DbPool>, auth: UserGuard) -> impl Responder {
    let user_id = match Uuid::parse_str(&auth.claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format in token"),
    };

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        user_service::get_user_profile(&mut conn, user_id)
    })
    .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("User not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_profile(
    pool: web::Data<DbPool>,
    auth: UserGuard,
    update_data: web::Json<UpdateUser>,
) -> impl Responder {
    let user_id = match Uuid::parse_str(&auth.claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format in token"),
    };
    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        user_service::update_user_profile(&mut conn, user_id, update_data.into_inner())
    })
    .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn delete_profile(pool: web::Data<DbPool>, auth: UserGuard) -> impl Responder {
    let user_id = match Uuid::parse_str(&auth.claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format in token"),
    };

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        user_service::soft_delete_user(&mut conn, user_id)
    })
    .await;

    match result {
        Ok(Ok(count)) if count > 0 => HttpResponse::NoContent().finish(),
        Ok(Ok(_)) => HttpResponse::NotFound().body("User not found"),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_users(pool: web::Data<DbPool> /*, auth: AdminGuard*/) -> impl Responder {
    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        user_service::get_all_users(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
