use axum::{Json, extract::{Path, State}, http::StatusCode};

use crate::app::RouteState;

pub async fn list_webhook_events(
    Path(webhook_id): Path<String>,
    State(state): State<RouteState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let events = state
        .store
        .list_events(
            &webhook_id,
            cornerback_core::store::Paging {
                limit: None,
                offset: None,
            },
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!(events)))
}
