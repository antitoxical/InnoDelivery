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
    courier_handler::{get_courier_profile, update_courier_status},
    user_handler::{delete_profile, get_profile, get_users, update_profile},
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
        web::scope("/api")
            .route("/profile", web::get().to(get_profile))
            .route("/update", web::patch().to(update_profile))
            .route("/delete", web::delete().to(delete_profile))
            .route("/users", web::get().to(get_users)),
    );
}

pub fn config_courier(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/courier")
            .route("/profile", web::get().to(get_courier_profile))
            .route("/status", web::patch().to(update_courier_status)),
    );
}
