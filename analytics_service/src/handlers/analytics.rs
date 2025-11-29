use axum::{extract::State, http::StatusCode, Json};
use crate::AppState;
use crate::models::event::{UserCreatedEvent, OrderFinishedEvent};
use crate::repository::analytics;

pub async fn track_user_registration(
    State(state): State<AppState>,
    Json(event): Json<UserCreatedEvent>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Received UserCreatedEvent: {:?}", event.user_id);

    analytics::save_user_event(&state.db, event)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save user event: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(StatusCode::OK)
}

pub async fn track_order_completion(
    State(state): State<AppState>,
    Json(event): Json<OrderFinishedEvent>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Received OrderFinishedEvent: {:?}", event.order_id);

    analytics::save_order_event(&state.db, event)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save order event: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(StatusCode::OK)
}