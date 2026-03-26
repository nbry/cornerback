use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::app::RouteState;

pub async fn get_event(
    Path(event_id): Path<uuid::Uuid>,
    State(state): State<RouteState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let event = state
        .store
        .get_event(event_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Event not found".into()))?;

    Ok(Json(serde_json::json!(event)))
}
