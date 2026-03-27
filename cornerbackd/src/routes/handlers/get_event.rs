use axum::{Json, extract::{Path, State}};

use crate::{app::RouteState, error::ApiError};

pub async fn get_event(
    Path(event_id): Path<uuid::Uuid>,
    State(state): State<RouteState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("fetching event: {}", event_id);

    let event = state
        .store
        .get_event(event_id)
        .await
        .map_err(|e| {
            tracing::error!("database error fetching event {}: {}", event_id, e);
            ApiError::internal_error(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("event not found: {}", event_id);
            ApiError::not_found("Event")
        })?;

    tracing::debug!("event retrieved: {}", event_id);
    Ok(Json(serde_json::json!(event)))
}
