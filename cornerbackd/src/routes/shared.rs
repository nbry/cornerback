use std::sync::atomic::Ordering;

use axum::Json;
use axum::http::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::app::RouteState;
use crate::error::ApiError;
use cornerback_core::models::{Event, NewEvent};

pub const X_CORNERBACK_REPLAY_HEADER: &str = "x-cornerback-replay";
pub const X_CORNERBACK_SEND_TARGET_HEADER: &str = "x-cornerback-send-target";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IncomingEventPayload {
    #[serde(default)]
    pub id: Option<uuid::Uuid>,
    #[serde(default = "default_json_object")]
    pub headers: serde_json::Value,
    pub body: serde_json::Value,
}

fn default_json_object() -> serde_json::Value {
    serde_json::json!({})
}

pub async fn process_event_request(
    state: &RouteState,
    webhook_id: String,
    request_headers: HeaderMap,
    payload: IncomingEventPayload,
) -> Result<Json<serde_json::Value>, ApiError> {
    tracing::info!("processing event for webhook: {}", webhook_id);

    // Proxy mode: intercept is off, so just forward and return immediately.
    if !state.intercept.load(Ordering::Relaxed) {
        tracing::debug!("proxy mode enabled, forwarding event directly");
        let event = Event::new(webhook_id.clone(), payload.headers, payload.body);
        send_event_to_target(state, &event).await?;
        return Ok(Json(serde_json::json!({ "proxied": true })));
    }

    let replay_requested = header_flag(&request_headers, X_CORNERBACK_REPLAY_HEADER);

    let event = if replay_requested {
        tracing::debug!("replay requested for webhook: {}", webhook_id);
        let event_id = payload.id.ok_or_else(|| {
            tracing::warn!("replay requested but no event id provided");
            ApiError::bad_request("id is required when replay header is set")
        })?;

        let mut event = state
            .store
            .get_event(event_id)
            .await
            .map_err(|e| {
                tracing::error!("failed to fetch event {} for replay: {}", event_id, e);
                ApiError::internal_error(e.to_string())
            })?
            .ok_or_else(|| {
                tracing::warn!("event {} not found for replay", event_id);
                ApiError::not_found("Event")
            })?;

        event = event.add_header(X_CORNERBACK_REPLAY_HEADER, "true");

        state
            .store
            .update_event(event.clone())
            .await
            .map_err(|e| {
                tracing::error!("failed to update event {} for replay: {}", event_id, e);
                ApiError::internal_error(e.to_string())
            })?;

        tracing::info!("event {} replayed for webhook: {}", event_id, webhook_id);
        event
    } else {
        let event = state
            .store
            .insert_event(NewEvent {
                webhook_id: webhook_id.clone(),
                headers: payload.headers,
                body: payload.body,
            })
            .await
            .map_err(|e| {
                tracing::error!("failed to insert event for webhook {}: {}", webhook_id, e);
                ApiError::internal_error(e.to_string())
            })?;
        tracing::info!("event {} inserted for webhook: {}", event.id, webhook_id);
        event
    };

    let send_to_target = state.config.send_to_target_by_default
        || header_flag(&request_headers, X_CORNERBACK_SEND_TARGET_HEADER);

    if send_to_target {
        send_event_to_target(state, &event).await?;
    }

    Ok(Json(serde_json::json!(event)))
}

pub fn header_flag(headers: &HeaderMap, key: &str) -> bool {
    headers
        .get(key)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on")
        })
        .unwrap_or(false)
}

pub async fn send_event_to_target(
    state: &RouteState,
    event: &Event,
) -> Result<(), ApiError> {
    tracing::debug!("forwarding event {} to target", event.id);

    let mut request = state.http_client.post(&state.config.target_server.url);

    for (k, v) in &state.config.target_server.headers {
        request = request.header(k, v);
    }

    let response = request
        .json(event)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("failed to send event {} to target: {}", event.id, e);
            ApiError::bad_gateway(format!("target send failed: {}", e))
        })?;

    if response.status().is_success() {
        tracing::info!("event {} successfully forwarded to target", event.id);
        Ok(())
    } else {
        let status = response.status();
        tracing::error!(
            "target rejected event {}: HTTP {}",
            event.id,
            status
        );
        Err(ApiError::bad_gateway(format!(
            "target returned status {}",
            status
        )))
    }
}
