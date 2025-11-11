use crate::models::courier::CourierStatus;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use uuid::Uuid;

pub async fn assign(_req: HttpRequest, pool: web::Data<crate::db::DbPool>) -> impl Responder {
    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;
        crate::repository::courier_repository::assign_any_free_courier(&mut conn)
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(Some(courier_id))) => {
            HttpResponse::Ok().json(serde_json::json!({"courier_id": courier_id}))
        }
        Ok(Ok(None)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn user_blocked(
    _req: HttpRequest,
    pool: web::Data<crate::db::DbPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();
    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;
        crate::repository::user_repository::is_user_blocked(&mut conn, user_id)
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(is_blocked)) => {
            HttpResponse::Ok().json(serde_json::json!({"is_blocked": is_blocked}))
        }
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
pub struct SetStatusBody {
    pub courier_id: Uuid,
}

pub async fn internal_set_courier_busy(
    _req: HttpRequest,
    pool: web::Data<crate::db::DbPool>,
    body: web::Json<SetStatusBody>,
) -> impl Responder {
    let id = body.courier_id;
    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;
        crate::repository::courier_repository::update_status_by_user_id(
            &mut conn,
            id,
            CourierStatus::Busy,
        )
        .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn internal_set_courier_free(
    _req: HttpRequest,
    pool: web::Data<crate::db::DbPool>,
    body: web::Json<SetStatusBody>,
) -> impl Responder {
    let id = body.courier_id;
    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;
        crate::repository::courier_repository::update_status_by_user_id(
            &mut conn,
            id,
            CourierStatus::Free,
        )
        .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn internal_update_courier_rating(
    _req: HttpRequest,
    pool: web::Data<crate::db::DbPool>,
    body: web::Json<SetCourierRatingBody>,
) -> impl Responder {
    let user_id = body.courier_id;
    let rating = body.rating;
    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;
        crate::repository::courier_repository::update_courier_rating(&mut conn, user_id, rating)
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => HttpResponse::InternalServerError().body(e),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(serde::Deserialize)]
pub struct SetCourierRatingBody {
    pub courier_id: Uuid,
    pub rating: f32,
}

pub async fn internal_set_courier_rating(
    _req: HttpRequest,
    pool: web::Data<crate::db::DbPool>,
    body: web::Json<SetCourierRatingBody>,
) -> impl Responder {
    let user_id = body.courier_id;
    let new_rating = body.rating;

    if !(1.0..=5.0).contains(&new_rating) {
        return HttpResponse::BadRequest().body("Rating must be between 1 and 5");
    }

    let result = web::block(move || {
        let mut conn = crate::db::get_conn_from_pool(&pool).map_err(|e| e.to_string())?;

        let _courier =
            crate::repository::courier_repository::find_courier_by_user_id(&mut conn, user_id)
                .map_err(|e| e.to_string())?;

        crate::repository::courier_repository::update_courier_rating(&mut conn, user_id, new_rating)
            .map_err(|e| e.to_string())
    })
    .await;

    match result {
        Ok(Ok(_)) => HttpResponse::NoContent().finish(),
        Ok(Err(e)) => {
            if e.contains("not found") {
                HttpResponse::NotFound().body("Courier not found")
            } else {
                HttpResponse::InternalServerError().body(e)
            }
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
