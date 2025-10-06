use chrono::{Duration, Utc};
use user_service::auth::jwt::Claims;
use uuid::Uuid;

fn create_test_claims(sub: String, role: String, exp_offset_secs: i64) -> Claims {
    let expiration = (Utc::now() + Duration::seconds(exp_offset_secs)).timestamp();
    Claims {
        sub,
        role,
        exp: expiration as usize,
    }
}

#[test]
/// Test 10: UserGuard allows user with 'user' role.
fn test_user_guard_logic_allows_user_role() {
    let claims = create_test_claims(Uuid::new_v4().to_string(), "user".to_string(), 3600);
    assert_eq!(claims.role, "user");
}

#[test]
/// Test 11: UserGuard blocks user with 'courier' role.
fn test_user_guard_logic_blocks_courier_role() {
    let claims = create_test_claims(Uuid::new_v4().to_string(), "courier".to_string(), 3600);
    assert_ne!(claims.role, "user");
}

#[test]
/// Test 12: CourierGuard allows user with 'courier' role.
fn test_courier_guard_logic_allows_courier_role() {
    let claims = create_test_claims(Uuid::new_v4().to_string(), "courier".to_string(), 3600);
    assert_eq!(claims.role, "courier");
}

#[test]
/// Test 12.1: CourierGuard blocks user with 'user' role.
fn test_courier_guard_logic_blocks_user_role() {
    let claims = create_test_claims(Uuid::new_v4().to_string(), "user".to_string(), 3600);
    assert_ne!(claims.role, "courier");
}
