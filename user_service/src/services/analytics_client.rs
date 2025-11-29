use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
pub struct UserCreatedEvent {
    pub user_id: Uuid,
    pub role: String,
    pub registered_at: DateTime<Utc>,
}

pub async fn notify_analytics_user_created(
    analytics_url: &str,
    user_id: Uuid,
    role: &str
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("{}/internal/analytics/user", analytics_url);

    let event = UserCreatedEvent {
        user_id,
        role: role.to_string(),
        registered_at: Utc::now(),
    };

    match client.post(&url).json(&event).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::error!("Analytics service returned error: {}", resp.status());
            } else {
                tracing::info!("Successfully notified analytics about new user");
            }
        }
        Err(e) => {
            tracing::error!("Failed to send event to analytics: {}", e);
        }
    }
    Ok(())
}