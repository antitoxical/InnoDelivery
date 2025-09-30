use crate::auth::guard::CourierGuard;
use crate::db;
use crate::db::DbPool;
use crate::models::courier::CourierStatus;
use crate::services::auth_service::AuthError;
use crate::services::courier_service;
use actix_web::{HttpResponse, Responder, web};
use uuid::Uuid;

pub async fn get_courier_profile(pool: web::Data<DbPool>, auth: CourierGuard) -> impl Responder {
    let user_id = match Uuid::parse_str(&auth.claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format in token"),
    };

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::get_courier_profile(&mut conn, user_id)
    })
    .await;

    match result {
        Ok(Ok(profile)) => HttpResponse::Ok().json(profile),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("Courier profile not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn update_courier_status(
    pool: web::Data<DbPool>,
    auth: CourierGuard,
    update_data: web::Json<String>,
) -> impl Responder {
    let new_status = match update_data.trim().to_lowercase().as_str() {
        "free" => CourierStatus::Free,
        "busy" => CourierStatus::Busy,
        _ => {
            return HttpResponse::BadRequest()
                .body("Invalid status value. Allowed values are 'Free' or 'Busy'.");
        }
    };
    let user_id = match Uuid::parse_str(&auth.claims.sub) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().body("Invalid user ID format in token"),
    };

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::update_courier_status(&mut conn, user_id, new_status)
    })
    .await;

    match result {
        Ok(Ok(updated_status)) => HttpResponse::Ok().json(updated_status),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("Courier profile not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
