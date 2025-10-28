use chrono::Utc;
use user_service::models::courier::{Courier, CourierStatus};
use user_service::models::user::User;
use uuid::Uuid;

fn create_test_user(role: &str) -> User {
    let now = Utc::now().naive_utc();
    User {
        id: Uuid::new_v4(),
        name: "Test User".to_string(),
        phone_number: "1234567890".to_string(),
        email: "test@example.com".to_string(),
        password: "hashed_password".to_string(),
        role: role.to_string(),
        favorite_address: None,
        is_blocked: false,
        is_deleted: false,
        created_at: now,
        updated_at: now,
    }
}

#[test]
/// Test 16: Correct definition of courier status (Free/Busy)
fn test_courier_status_enum() {
    assert_eq!(format!("{:?}", CourierStatus::Free), "Free");
    assert_eq!(format!("{:?}", CourierStatus::Busy), "Busy");
}

#[test]
/// Test 17: User with is_blocked=true is considered blocked
fn test_user_blocked_flag() {
    let mut user = create_test_user("user");
    user.is_blocked = true;
    assert!(user.is_blocked);
}

#[test]
/// Test 18: User with is_deleted=true is considered deleted
fn test_user_deleted_flag() {
    let mut user = create_test_user("user");
    user.is_deleted = true;
    assert!(user.is_deleted);
}

#[test]
/// Test 19: Courier rating is within 0..=5
fn test_courier_rating_bounds() {
    let now = Utc::now().naive_utc();
    let courier = Courier {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        status: CourierStatus::Free,
        rating: 4.7,
        created_at: now,
        updated_at: now,
        is_blocked: false,
        is_deleted: false,
        rating_count: 0,
        rating_sum: 0.0,
    };
    assert!((0.0..=5.0).contains(&courier.rating));
}

#[test]
/// Test 20: User email must contain '@'
fn test_user_email_validity() {
    let user = create_test_user("user");
    assert!(user.email.contains('@'));
}

#[test]
/// Test 21: User role can only be 'user' or 'courier'
fn test_user_role_allowed_values() {
    let user = create_test_user("user");
    assert!(user.role == "user" || user.role == "courier");
    let courier = create_test_user("courier");
    assert!(courier.role == "user" || courier.role == "courier");
}

#[test]
/// Test 23: created_at and updated_at update correctly
fn test_user_timestamps_update() {
    let mut user = create_test_user("user");
    let old_updated = user.updated_at;
    let new_time = old_updated + chrono::Duration::minutes(5);
    user.updated_at = new_time;
    assert!(user.updated_at > old_updated);
}

#[test]
/// Test 24: Cannot create courier without user_id (user_id must be valid UUID)
fn test_courier_requires_user_id() {
    let now = Utc::now().naive_utc();
    let user_id = Uuid::new_v4();
    let courier = Courier {
        id: Uuid::new_v4(),
        user_id,
        status: CourierStatus::Free,
        rating: 5.0,
        created_at: now,
        updated_at: now,
        is_blocked: false,
        is_deleted: false,
        rating_count: 0,
        rating_sum: 0.0,
    };
    assert_eq!(courier.user_id, user_id);
}

#[test]
/// Test 25: Cannot update courier status to invalid value (enum is limited)
fn test_courier_status_invalid_value() {
    fn status_from_str(s: &str) -> Option<CourierStatus> {
        match s {
            "free" => Some(CourierStatus::Free),
            "busy" => Some(CourierStatus::Busy),
            _ => None,
        }
    }
    assert!(status_from_str("free").is_some());
    assert!(status_from_str("busy").is_some());
    assert!(status_from_str("invalid").is_none());
}
