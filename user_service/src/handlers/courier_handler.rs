use crate::auth::guard::{AdminGuard, CourierGuard};
use crate::db;
use crate::db::DbPool;
use crate::dto::user_dto::UpdateUser;
use crate::models::courier::CourierStatus;
use crate::services::auth_service::AuthError;
use crate::services::{courier_service, user_service};
use actix_web::{HttpResponse, Responder, web};
use uuid::Uuid;
use validator::Validate;

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

pub async fn get_all_couriers(pool: web::Data<DbPool>, _auth: AdminGuard) -> impl Responder {
    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::get_all_couriers(&mut conn)
    })
    .await;

    match result {
        Ok(Ok(profiles)) => HttpResponse::Ok().json(profiles),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn block_courier(
    pool: web::Data<DbPool>,
    _auth: AdminGuard,
    path: web::Path<Uuid>,
) -> impl Responder {
    let courier_user_id = path.into_inner();

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::set_courier_blocked_status(&mut conn, courier_user_id, true)
    })
    .await;

    match result {
        Ok(Ok(courier)) => HttpResponse::Ok().json(courier),
        Ok(Err(e)) => match e {
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("Courier not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn unblock_courier(
    pool: web::Data<DbPool>,
    _auth: AdminGuard,
    path: web::Path<Uuid>,
) -> impl Responder {
    let courier_user_id = path.into_inner();

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::set_courier_blocked_status(&mut conn, courier_user_id, false)
    })
    .await;

    match result {
        Ok(Ok(courier)) => HttpResponse::Ok().json(courier),
        Ok(Err(e)) => match e {
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("Courier not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn admin_update_courier(
    pool: web::Data<DbPool>,
    _auth: AdminGuard,
    path: web::Path<Uuid>,
    update_data: web::Json<UpdateUser>,
) -> impl Responder {
    if let Err(validation_errors) = update_data.validate() {
        return HttpResponse::BadRequest().json(validation_errors);
    }
    let courier_user_id = path.into_inner();

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        user_service::update_user_profile(&mut conn, courier_user_id, update_data.into_inner())
    })
    .await;

    match result {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(e)) => match e {
            AuthError::DatabaseError(msg) if msg.contains("NotFound") => {
                HttpResponse::NotFound().body("Courier(user) not found")
            }
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn admin_delete_courier(
    pool: web::Data<DbPool>,
    _auth: AdminGuard,
    path: web::Path<Uuid>,
) -> impl Responder {
    let courier_user_id = path.into_inner();

    let result = web::block(move || {
        let mut conn = match db::get_conn_from_pool(&pool) {
            Ok(connection) => connection,
            Err(e) => return Err(AuthError::ConnectionError(e.to_string())),
        };
        courier_service::soft_delete_courier(&mut conn, courier_user_id)
    })
    .await;

    match result {
        Ok(Ok(count)) if count > 0 => HttpResponse::NoContent().finish(),
        Ok(Ok(_)) => HttpResponse::NotFound().body("Courier not found"),
        Ok(Err(e)) => match e {
            AuthError::ConnectionError(msg) => HttpResponse::ServiceUnavailable().body(msg),
            _ => HttpResponse::InternalServerError().body(e.to_string()),
        },
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
