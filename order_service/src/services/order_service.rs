use crate::config;
use crate::db::DbPool;
use crate::models::{NewOrder, Order, OrderStatus};
use crate::repository::order_repository::{self, OrderProductData};
use log;
use reqwest;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum OrderServiceError {
    DatabaseError(String),
    UserServiceError(String),
    InvalidInput(String),
    RatingWindowExpired,
    InvalidRating,
}

impl fmt::Display for OrderServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderServiceError::DatabaseError(e) => write!(f, "Database error: {}", e),
            OrderServiceError::UserServiceError(e) => write!(f, "User service error: {}", e),
            OrderServiceError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
            OrderServiceError::RatingWindowExpired => write!(f, "Rating window has expired"),
            OrderServiceError::InvalidRating => write!(f, "Invalid rating value"),
        }
    }
}

impl From<diesel::result::Error> for OrderServiceError {
    fn from(error: diesel::result::Error) -> Self {
        OrderServiceError::DatabaseError(error.to_string())
    }
}

impl From<reqwest::Error> for OrderServiceError {
    fn from(error: reqwest::Error) -> Self {
        OrderServiceError::UserServiceError(error.to_string())
    }
}

pub async fn rate_order(
    pool: &DbPool,
    user_service_url: &str,
    order_id: Uuid,
    user_id: Uuid,
    rating: f32,
    rating_window_minutes: i32,
) -> Result<Order, OrderServiceError> {
    if !(1.0..=5.0).contains(&rating) {
        return Err(OrderServiceError::InvalidRating);
    }

    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;

    let order = order_repository::find_order_by_id(&mut conn, order_id)?;

    if order.user_id != user_id {
        return Err(OrderServiceError::InvalidInput(
            "Order not found".to_string(),
        ));
    }

    let updated_order =
        order_repository::rate_order_in_window(&mut conn, order_id, rating, rating_window_minutes)
            .map_err(|e| {
                if e == diesel::result::Error::NotFound
                    || e == diesel::result::Error::RollbackTransaction
                {
                    OrderServiceError::RatingWindowExpired
                } else {
                    OrderServiceError::DatabaseError(e.to_string())
                }
            })?;

    if let Some(courier_id) = updated_order.courier_id {
        let client = reqwest::Client::new();
        let url = format!("{}/internal/couriers/set_rating", user_service_url);
        let body = serde_json::json!({
            "courier_id": courier_id,
            "rating": rating
        });

        match client.post(&url).json(&body).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("Successfully updated courier rating in user_service");
                } else {
                    log::error!(
                        "Failed to update courier rating: HTTP {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                log::error!("Error calling user_service to update rating: {}", e);
            }
        }
    }

    Ok(updated_order)
}

pub async fn create_order(
    pool: &DbPool,
    user_service_url: &str,
    user_id: Uuid,
    delivery_address: &str,
    products: Vec<OrderProductData>,
) -> Result<Order, OrderServiceError> {
    log::info!(
        "[CREATE_ORDER] Called with user_id={}, delivery_address={}, products={:?}",
        user_id,
        delivery_address,
        products
    );

    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;

    let product_ids: Vec<Uuid> = products.iter().map(|p| p.product_id).collect();
    let missing = order_repository::find_missing_product_ids(&mut conn, &product_ids)?;
    if !missing.is_empty() {
        log::warn!("[CREATE_ORDER] Missing products: {:?}", missing);
        return Err(OrderServiceError::InvalidInput(format!(
            "Unknown product ids: {:?}",
            missing
        )));
    }

    if is_user_blocked(user_service_url, user_id).await? {
        // <--- Передаем URL
        log::warn!("[CREATE_ORDER] User is blocked: {}", user_id);
        return Err(OrderServiceError::InvalidInput(
            "User is blocked".to_string(),
        ));
    }

    let courier_id_option = try_assign_courier(user_service_url).await?;
    log::debug!(
        "[CREATE_ORDER] Courier assignment result: {:?}",
        courier_id_option
    );
    let order_status;
    let final_courier_id;

    if let Some(courier_id) = courier_id_option {
        let busy_result = set_courier_busy(user_service_url, courier_id).await;
        log::debug!("[CREATE_ORDER] set_courier_busy result: {:?}", busy_result);
        if busy_result.is_ok() {
            order_status = OrderStatus::InProgress;
            final_courier_id = Some(courier_id);
        } else {
            log::warn!("[CREATE_ORDER] set_courier_busy failed, courier will not be assigned");
            let _ = release_courier(user_service_url, courier_id).await;
            order_status = OrderStatus::PendingCarrier;
            final_courier_id = None;
        }
    } else {
        log::info!("[CREATE_ORDER] No courier assigned, status PendingCarrier");
        order_status = OrderStatus::PendingCarrier;
        final_courier_id = None;
    }

    let new_order = NewOrder {
        user_id,
        courier_id: final_courier_id,
        delivery_address: delivery_address.to_string(),
        status: order_status,
    };
    log::info!("[CREATE_ORDER] NewOrder struct: {:?}", new_order);

    let result = order_repository::create_order(&mut conn, new_order, products);
    match &result {
        Ok(order) => {
            log::info!(
                "[CREATE_ORDER] Order successfully created! id={:?}",
                order.id
            );
        }
        Err(e) => {
            log::error!("[CREATE_ORDER] Error during order creation: {:?}", e);
        }
    }
    result.map_err(OrderServiceError::from)
}

pub async fn set_courier_busy(
    user_service_url: &str,
    courier_id: Uuid,
) -> Result<(), OrderServiceError> {
    let url = format!("{}/internal/couriers/busy", user_service_url);
    let body = serde_json::json!({
        "courier_id": courier_id
    });
    reqwest::Client::new()
        .post(&url)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

pub async fn is_user_blocked(
    user_service_url: &str,
    user_id: Uuid,
) -> Result<bool, OrderServiceError> {
    let url = format!("{}/internal/users/{}/blocked", user_service_url, user_id);
    log::debug!(" Sending request to: {}", url);

    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;

    log::debug!(" Received response status: {}", resp.status());

    if resp.status().is_success() {
        let body = resp.text().await?;
        log::debug!(" Response body: {}", body);

        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            OrderServiceError::UserServiceError(format!("Failed to parse JSON: {}", e))
        })?;

        Ok(v.get("is_blocked")
            .and_then(|b| b.as_bool())
            .unwrap_or(false))
    } else {
        log::error!(" Request failed with status: {}", resp.status());
        Err(OrderServiceError::UserServiceError(format!(
            "User service returned error status: {}",
            resp.status()
        )))
    }
}

pub async fn try_assign_courier(user_service_url: &str) -> Result<Option<Uuid>, OrderServiceError> {
    let url = format!("{}/internal/couriers/assign", user_service_url);
    log::debug!(" Sending courier assignment request to: {}", url);

    let client = reqwest::Client::new();
    let resp = client.post(&url).send().await?;

    log::debug!(" Received response status: {}", resp.status());

    if resp.status().as_u16() == 204 {
        log::debug!(" No courier available (204 No Content)");
        return Ok(None);
    }

    if resp.status().is_success() {
        let body = resp.text().await?;
        log::debug!(" Response body: {}", body);

        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            OrderServiceError::UserServiceError(format!("Failed to parse JSON: {}", e))
        })?;

        let id = v
            .get("courier_id")
            .and_then(|s| s.as_str())
            .and_then(|s| Uuid::parse_str(s).ok());

        log::debug!(" Parsed courier_id: {:?}", id);
        Ok(id)
    } else {
        let status = resp.status();
        let body = resp.text().await.map_err(|e| {
            OrderServiceError::UserServiceError(format!("Failed to get response body: {}", e))
        })?;
        let error_msg = format!(
            "User service returned unexpected status: {} - Body: {}",
            status, body
        );
        log::error!("{}", error_msg);
        Err(OrderServiceError::UserServiceError(error_msg))
    }
}
pub async fn release_courier(
    user_service_url: &str,
    courier_id: Uuid,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/internal/couriers/free", user_service_url);
    let body = serde_json::json!({"courier_id": courier_id});
    let resp = reqwest::Client::new().post(&url).json(&body).send().await?;

    if resp.status().is_success() {
        Ok(())
    } else {
        Err(resp.error_for_status().unwrap_err())
    }
}

pub async fn complete_order(
    pool: &DbPool,
    user_service_url: &str,
    order_id: Uuid,
) -> Result<Order, OrderServiceError> {
    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;

    let order = order_repository::find_order_by_id(&mut conn, order_id)?;
    if let Some(courier_id) = order.courier_id {
        release_courier(user_service_url, courier_id).await?;
    }
    order_repository::update_order_status(&mut conn, order_id, OrderStatus::Finished)
        .map_err(OrderServiceError::from)?;
    let updated_order = order_repository::find_order_by_id(&mut conn, order_id)?;
    Ok(updated_order)
}

pub async fn process_pending_orders(
    pool: &DbPool,
    user_service_url: &str,
    timeout_seconds: i64,
) -> Result<usize, OrderServiceError> {
    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;

    let pending_orders = order_repository::find_expired_pending_orders(
        &mut conn,
        chrono::Utc::now().naive_utc() - chrono::Duration::seconds(timeout_seconds),
    )?;

    let mut processed = 0;

    for order in pending_orders {
        if let Some(courier_id) = try_assign_courier(user_service_url).await? {
            // <--- Передаем URL
            order_repository::update_order_status_and_courier(
                &mut conn,
                order.id,
                OrderStatus::InProgress,
                Some(courier_id),
            )?;
        } else {
            order_repository::update_order_status(&mut conn, order.id, OrderStatus::Cancelled)?;
        }

        processed += 1;
    }

    Ok(processed)
}

pub async fn cancel_order_wrapper(
    pool: &DbPool,
    order_id: Uuid,
) -> Result<Order, OrderServiceError> {
    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;
    order_repository::update_order_status(&mut conn, order_id, OrderStatus::Cancelled)
        .map_err(OrderServiceError::from)?;
    let updated_order = order_repository::find_order_by_id(&mut conn, order_id)?;
    Ok(updated_order)
}

pub async fn update_order_address(
    pool: &DbPool,
    order_id: Uuid,
    new_address: &str,
) -> Result<Order, OrderServiceError> {
    log::info!(
        "[UPDATE_ADDRESS] Called for order_id={} with new_address={}",
        order_id,
        new_address
    );

    let mut conn = pool
        .get()
        .map_err(|e| OrderServiceError::DatabaseError(e.to_string()))?;

    let updated_order = order_repository::update_order_address(&mut conn, order_id, new_address)?;

    log::info!(
        "[UPDATE_ADDRESS] Successfully updated address for order_id={}",
        order_id
    );

    Ok(updated_order)
}
