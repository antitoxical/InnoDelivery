use lazy_static::lazy_static;
use std::borrow::ToOwned;
use std::env::var;

lazy_static! {
    pub static ref DATABASE_URL: String = var("DATABASE_URL").expect("DATABASE_URL must be set");
    pub static ref JWT_SECRET: String = var("JWT_SECRET").expect("JWT_SECRET must be set");

}
