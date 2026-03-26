use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
};

use crate::app::RouteState;
use crate::routes::shared::{
    IncomingEventPayload, X_CORNERBACK_REPLAY_HEADER, process_event_request,
};

pub async fn replay_event(
    Path(event_id): Path<uuid::Uuid>,
    State(state): State<RouteState>,
    mut request_headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let event = state
        .store
        .get_event(event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Event not found".into()))?;

    let replay_payload = IncomingEventPayload {
        id: Some(event.id),
        headers: event.headers,
        body: event.body,
    };

    request_headers.insert(X_CORNERBACK_REPLAY_HEADER, HeaderValue::from_static("true"));

    process_event_request(&state, event.webhook_id, request_headers, replay_payload).await
}
