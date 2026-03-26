use axum::{Json, extract::{Path, State}, http::{HeaderMap, StatusCode}};

use crate::app::RouteState;
use crate::routes::shared::{IncomingEventPayload, process_event_request};

pub async fn handle_event(
    Path(webhook_id): Path<String>,
    State(state): State<RouteState>,
    request_headers: HeaderMap,
    Json(payload): Json<IncomingEventPayload>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    process_event_request(&state, webhook_id, request_headers, payload).await
}
