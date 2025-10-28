use crate::db::{DbPool, get_conn_from_pool};
use crate::models::user::NewUser as DbNewUser;
use crate::repository::user_repository;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString},
};
use diesel::result::Error::NotFound;
use rand::thread_rng;
use std::env;

pub async fn init_admin(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = get_conn_from_pool(pool)?;

    let admin_email = env::var("ADMIN_EMAIL").expect("ADMIN_EMAIL must be set");
    let admin_password = env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD must be set");
    let admin_phone = env::var("ADMIN_PHONE").expect("ADMIN_PHONE must be set");
    let admin_name = env::var("ADMIN_NAME").unwrap_or_else(|_| "admin".to_string());

    match user_repository::find_by_phone(&mut conn, &admin_phone) {
        Ok(_) => {
            tracing::info!("Admin user with phone {} already exists.", admin_phone);
        }
        Err(NotFound) => {
            tracing::info!("Admin user not found, creating a new one.");
            let salt = SaltString::generate(&mut thread_rng());
            let argon2 = Argon2::default();
            let hashed_password = argon2
                .hash_password(admin_password.as_bytes(), &salt)
                .map_err(|e| format!("argon2 error: {e}"))?
                .to_string();

            let new_admin = DbNewUser {
                name: admin_name,
                phone_number: admin_phone,
                email: admin_email,
                password: hashed_password,
                role: "admin".to_string(),
            };

            user_repository::create(&mut conn, &new_admin)?;
            tracing::info!("Admin user created successfully.");
        }
        Err(e) => {
            return Err(Box::new(e));
        }
    }

    Ok(())
}
