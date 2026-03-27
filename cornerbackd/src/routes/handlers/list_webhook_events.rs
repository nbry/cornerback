use axum::{Json, extract::{Path, State}};

use crate::{app::RouteState, error::ApiError};

pub async fn list_webhook_events(
    Path(webhook_id): Path<String>,
    State(state): State<RouteState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("listing events for webhook: {}", webhook_id);

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
        .map_err(|e| {
            tracing::error!("database error listing events for webhook {}: {}", webhook_id, e);
            ApiError::internal_error(e.to_string())
        })?;

    tracing::debug!("retrieved {} events for webhook: {}", events.len(), webhook_id);
    Ok(Json(serde_json::json!(events)))
}
