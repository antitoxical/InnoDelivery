use lazy_static::lazy_static;
use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        dotenv::dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("DATABASE_URL must be set");

        Config {
            database_url,
            jwt_secret,
        }
    };
}

use crate::handlers::{
    auth_handler::{login_user, register_user},
    courier_handler::{
        admin_delete_courier, admin_update_courier, block_courier, get_all_couriers,
        get_courier_profile, unblock_courier, update_courier_status,
    },
    user_handler::{
        admin_delete_user, admin_update_user, block_user, delete_profile, get_profile, get_users,
        unblock_user, update_profile,
    },
};
use actix_web::web;

pub fn config_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(register_user))
            .route("/login", web::post().to(login_user)),
    );
}

pub fn config_user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/profile", web::get().to(get_profile))
            .route("/update", web::put().to(update_profile))
            .route("/delete", web::delete().to(delete_profile)),
    );
}

pub fn config_courier(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/courier")
            .route("/profile", web::get().to(get_courier_profile))
            .route("/status", web::patch().to(update_courier_status)),
    );
}

pub fn config_admin(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users_admin")
            .route("/all", web::get().to(get_users))
            .route("/block/{id}", web::patch().to(block_user))
            .route("/unblock/{id}", web::patch().to(unblock_user))
            .route("/{id}", web::put().to(admin_update_user))
            .route("/{id}", web::delete().to(admin_delete_user)),
    )
    .service(
        web::scope("/couriers_admin")
            .route("/all", web::get().to(get_all_couriers))
            .route("/block/{id}", web::patch().to(block_courier))
            .route("/unblock/{id}", web::patch().to(unblock_courier)), //.route("/{id}", web::put().to(admin_update_courier))
                                                                       //.route("/{id}", web::delete().to(admin_delete_courier)),
    );
}
