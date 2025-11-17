use httpmock::Method::{GET, POST};
use httpmock::MockServer;
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
