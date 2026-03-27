use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
};

use crate::app::RouteState;
use crate::error::ApiError;
use crate::routes::shared::{
    IncomingEventPayload, X_CORNERBACK_REPLAY_HEADER, process_event_request,
};

pub async fn replay_event(
    Path(event_id): Path<uuid::Uuid>,
    State(state): State<RouteState>,
    mut request_headers: HeaderMap,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("replay requested for event: {}", event_id);

    let event = state
        .store
        .get_event(event_id)
        .await
        .map_err(|e| {
            tracing::error!("failed to fetch event {} for replay: {}", event_id, e);
            ApiError::internal_error(e.to_string())
        })?
        .ok_or_else(|| {
            tracing::warn!("event {} not found", event_id);
            ApiError::not_found("Event")
        })?;

    let replay_payload = IncomingEventPayload {
        id: Some(event.id),
        headers: event.headers,
        body: event.body,
    };

    request_headers.insert(X_CORNERBACK_REPLAY_HEADER, HeaderValue::from_static("true"));

    tracing::debug!("replaying event {} for webhook: {}", event_id, event.webhook_id);
    process_event_request(&state, event.webhook_id, request_headers, replay_payload).await
}
