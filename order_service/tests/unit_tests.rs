use async_graphql::{ErrorExtensions, Value};
use diesel::result::Error as DieselError;
use httpmock::Method::{GET, POST};
use httpmock::MockServer;
use order_service::graphql::GraphQLError;
use order_service::services::order_service::OrderServiceError;
use order_service::services::order_service::{
    is_user_blocked, release_courier, set_courier_busy, try_assign_courier,
};
use uuid::Uuid;

#[tokio::test]
async fn try_assign_returns_none_on_204() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(204);
    });
    let res = try_assign_courier(&server.base_url()).await.unwrap();
    assert!(res.is_none());

    mock.assert();
}

#[tokio::test]
async fn try_assign_parses_uuid_on_200() {
    let server = MockServer::start_async().await;
    let id = Uuid::new_v4();
    let body = serde_json::json!({ "courier_id": id.to_string() }).to_string();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .header("content-type", "application/json")
            .body(body);
    });
    let res = try_assign_courier(&server.base_url()).await.unwrap();
    assert_eq!(res, Some(id));

    mock.assert();
}

#[tokio::test]
async fn try_assign_malformed_json_returns_none() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .header("content-type", "application/json")
            .body("{not json}");
    });
    let res = try_assign_courier(&server.base_url()).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn try_assign_invalid_uuid_returns_none() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .header("content-type", "application/json")
            .body(serde_json::json!({ "courier_id": "not-a-uuid" }).to_string());
    });
    let res = try_assign_courier(&server.base_url()).await.unwrap();
    assert!(res.is_none());

    mock.assert();
}

#[tokio::test]
async fn try_assign_missing_field_returns_none() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200)
            .header("content-type", "application/json")
            .body(serde_json::json!({ "foo": "bar" }).to_string());
    });
    let res = try_assign_courier(&server.base_url()).await.unwrap();
    assert!(res.is_none());

    mock.assert();
}

#[tokio::test]
async fn try_assign_server_error_returns_none() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(500).body("err");
    });

    let res = try_assign_courier(&server.base_url()).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn try_assign_empty_body_returns_none() {
    let server = MockServer::start_async().await;
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200).body("");
    });
    let res = try_assign_courier(&server.base_url()).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn try_assign_extra_fields_ok() {
    let server = MockServer::start_async().await;
    let id = Uuid::new_v4();
    let body = serde_json::json!({ "courier_id": id.to_string(), "extra": 123 }).to_string();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/assign");
        then.status(200).body(body);
    });
    let res = try_assign_courier(&server.base_url()).await.unwrap();
    assert_eq!(res, Some(id));

    mock.assert();
}

#[tokio::test]
async fn try_assign_network_error() {
    let url = "http://127.0.0.1:1".to_string();
    let res = try_assign_courier(&url).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn is_user_blocked_true() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"is_blocked":true}"#);
    });
    let res = is_user_blocked(&server.base_url(), uid).await.unwrap();
    assert!(res);

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_false() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"is_blocked":false}"#);
    });
    let res = is_user_blocked(&server.base_url(), uid).await.unwrap();
    assert!(!res);

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_missing_field_false() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"other_field":123}"#);
    });
    let res = is_user_blocked(&server.base_url(), uid).await.unwrap();
    assert!(!res);

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_non_200_false() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(404);
    });

    let res = is_user_blocked(&server.base_url(), uid).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_malformed_json_false() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(200).body("invalid json");
    });

    let res = is_user_blocked(&server.base_url(), uid).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_whitespace_body_false() {
    let server = MockServer::start_async().await;
    let uid = Uuid::new_v4();
    let path = format!("/internal/users/{}/blocked", uid);
    let mock = server.mock(|when, then| {
        when.method(GET).path(path.as_str());
        then.status(200).body("   ");
    });
    let res = is_user_blocked(&server.base_url(), uid).await;
    assert!(res.is_err());

    mock.assert();
}

#[tokio::test]
async fn is_user_blocked_network_error() {
    let url = "http://127.0.0.1:1".to_string();
    let uid = Uuid::new_v4();
    let res = is_user_blocked(&url, uid).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn set_courier_busy_ok_on_200() {
    let server = MockServer::start_async().await;
    let cid = Uuid::new_v4();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/internal/couriers/busy")
            .header("content-type", "application/json")
            .json_body(serde_json::json!({"courier_id": cid}));
        then.status(200);
    });
    let res = set_courier_busy(&server.base_url(), cid).await;
    mock.assert();
    assert!(res.is_ok());
}

#[tokio::test]
async fn set_courier_busy_err_on_non_200() {
    let server = MockServer::start_async().await;
    let cid = Uuid::new_v4();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/busy");
        then.status(404);
    });
    let res = set_courier_busy(&server.base_url(), cid).await;
    mock.assert();
    assert!(res.is_err());
}

#[tokio::test]
async fn release_courier_ok_on_200() {
    let server = MockServer::start_async().await;
    let cid = Uuid::new_v4();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/internal/couriers/free")
            .header("content-type", "application/json")
            .json_body(serde_json::json!({"courier_id": cid}));
        then.status(200);
    });
    let res = release_courier(&server.base_url(), cid).await;
    mock.assert();
    assert!(res.is_ok());
}

#[tokio::test]
async fn release_courier_err_on_non_200() {
    let server = MockServer::start_async().await;
    let cid = Uuid::new_v4();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/internal/couriers/free");
        then.status(500).body("Internal Server Error");
    });
    let res = release_courier(&server.base_url(), cid).await;
    mock.assert();
    assert!(res.is_err());
}

#[test]
fn test_diesel_other_error_converts_to_database_error() {
    let diesel_err = DieselError::RollbackTransaction;
    let service_err: OrderServiceError = diesel_err.into();

    assert!(matches!(service_err, OrderServiceError::DatabaseError(_)));
    assert!(service_err.to_string().contains("Database error"));
}

#[test]
fn test_graphql_display_implementation() {
    let err = GraphQLError::NotFound;
    assert_eq!(format!("{}", err), "Resource not found");

    let err = GraphQLError::InvalidInput("empty list".to_string());
    assert_eq!(format!("{}", err), "Invalid input: empty list");
}

#[test]
fn test_graphql_from_conversions() {
    let conn_error = diesel::ConnectionError::BadConnection("Test connection error".to_string());

    let r2d2_err = diesel::r2d2::Error::ConnectionError(conn_error);

    let graphql_err: GraphQLError = r2d2_err.into();

    assert!(matches!(graphql_err, GraphQLError::ConnectionError(_)));

    if let GraphQLError::ConnectionError(msg) = graphql_err {
        assert!(msg.contains("Test connection error"));
    }

    let async_err = async_graphql::Error::new("Async error");
    let graphql_err: GraphQLError = async_err.into();
    assert!(matches!(graphql_err, GraphQLError::InvalidInput(_)));
}

#[test]
fn test_graphql_error_extensions_codes() {
    let db_err = GraphQLError::DatabaseError("Db fail".to_string());
    let field_error = db_err.extend();

    let extensions = field_error
        .extensions
        .as_ref()
        .expect("Extensions should exist");

    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("DATABASE_ERROR".to_string()))
    );
    assert_eq!(
        extensions.get("details"),
        Some(&Value::String("Db fail".to_string()))
    );

    let conn_err = GraphQLError::ConnectionError("Pool fail".to_string());
    let field_error = conn_err.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("DB_CONNECTION_ERROR".to_string()))
    );

    let not_found = GraphQLError::NotFound;
    let field_error = not_found.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("NOT_FOUND".to_string()))
    );
    assert_eq!(
        extensions.get("details"),
        Some(&Value::String("Resource not found".to_string()))
    );

    let invalid = GraphQLError::InvalidInput("bad data".to_string());
    let field_error = invalid.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("INVALID_INPUT".to_string()))
    );
    assert_eq!(
        extensions.get("details"),
        Some(&Value::String("bad data".to_string()))
    );

    let service_err =
        GraphQLError::ServiceError(OrderServiceError::InvalidInput("Bad data".to_string()));
    let field_error = service_err.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("INVALID_INPUT".to_string()))
    );

    let expired = GraphQLError::ServiceError(OrderServiceError::RatingWindowExpired);
    let field_error = expired.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("RATING_WINDOW_EXPIRED".to_string()))
    );

    let invalid_rating = GraphQLError::ServiceError(OrderServiceError::InvalidRating);
    let field_error = invalid_rating.extend();
    let extensions = field_error.extensions.as_ref().unwrap();
    assert_eq!(
        extensions.get("code"),
        Some(&Value::String("INVALID_RATING".to_string()))
    );
}

#[test]
fn test_order_service_error_display() {
    let err = OrderServiceError::DatabaseError("db fail".to_string());
    assert_eq!(format!("{}", err), "Database error: db fail");

    let err = OrderServiceError::UserServiceError("http fail".to_string());
    assert_eq!(format!("{}", err), "User service error: http fail");

    let err = OrderServiceError::InvalidInput("bad input".to_string());
    assert_eq!(format!("{}", err), "Invalid input: bad input");

    let err = OrderServiceError::RatingWindowExpired;
    assert_eq!(format!("{}", err), "Rating window has expired");

    let err = OrderServiceError::InvalidRating;
    assert_eq!(format!("{}", err), "Invalid rating value");
}

#[test]
fn test_diesel_not_found_converts_to_service_not_found() {
    let diesel_err = DieselError::NotFound;
    let service_err: OrderServiceError = diesel_err.into();

    assert!(matches!(service_err, OrderServiceError::DatabaseError(_)));

    assert_eq!(
        format!("{}", service_err),
        "Database error: Record not found"
    );
}

#[test]
fn test_graphql_from_diesel_error_general() {
    let custom_msg = "Diesel error";
    let diesel_err = DieselError::QueryBuilderError(custom_msg.into());

    let graphql_err: GraphQLError = diesel_err.into();

    assert!(matches!(graphql_err, GraphQLError::DatabaseError(_)));

    if let GraphQLError::DatabaseError(msg) = graphql_err {
        assert!(
            msg.contains(custom_msg),
            "Expected message to contain '{}', but got '{}'",
            custom_msg,
            msg
        );
    }
}

#[test]
fn test_graphql_not_found_display_and_conversion() {
    let diesel_err = diesel::result::Error::NotFound;
    let graphql_err: GraphQLError = diesel_err.into();

    assert!(matches!(graphql_err, GraphQLError::NotFound));
    assert_eq!(format!("{}", graphql_err), "Resource not found");
}
#[test]
fn test_graphql_from_service_error() {
    let service_err = OrderServiceError::InvalidRating;
    let graphql_err: GraphQLError = service_err.into();

    assert!(matches!(graphql_err, GraphQLError::ServiceError(_)));
}
#[test]
fn test_graphql_database_error_conversion() {
    let diesel_err = diesel::result::Error::RollbackTransaction;
    let graphql_err: GraphQLError = diesel_err.into();

    assert!(matches!(graphql_err, GraphQLError::DatabaseError(_)));
}

#[test]
fn test_graphql_service_error_fallback_extension() {
    let internal_err = OrderServiceError::DatabaseError("Some DB issue".to_string());
    let graphql_err = GraphQLError::ServiceError(internal_err);

    let field_error = graphql_err.extend();
    let extensions = field_error
        .extensions
        .as_ref()
        .expect("Extensions should exist");
    let code = extensions.get("code");

    assert_eq!(code, Some(&Value::String("SERVICE_ERROR".to_string())));
}
