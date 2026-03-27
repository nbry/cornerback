use std::sync::atomic::Ordering;

use axum::{
    Json,
    http::{HeaderMap, StatusCode},
};
use serde::{Deserialize, Serialize};

use crate::app::RouteState;
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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Proxy mode: intercept is off, so just forward and return immediately.
    if !state.intercept.load(Ordering::Relaxed) {
        let event = Event::new(webhook_id, payload.headers, payload.body);
        send_event_to_target(state, &event).await?;
        return Ok(Json(serde_json::json!({ "proxied": true })));
    }

    let replay_requested = header_flag(&request_headers, X_CORNERBACK_REPLAY_HEADER);

    let event = if replay_requested {
        let event_id = payload.id.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "id is required when replay header is set".to_string(),
            )
        })?;

        let mut event = state
            .store
            .get_event(event_id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or_else(|| (StatusCode::NOT_FOUND, "Event not found".into()))?;

        event = event.add_header(X_CORNERBACK_REPLAY_HEADER, "true");

        state
            .store
            .update_event(event.clone())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        event
    } else {
        let event = state
            .store
            .insert_event(NewEvent {
                webhook_id,
                headers: payload.headers,
                body: payload.body,
            })
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
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
) -> Result<(), (StatusCode, String)> {
    let mut request = state.http_client.post(&state.config.target_server.url);

    for (k, v) in &state.config.target_server.headers {
        request = request.header(k, v);
    }

    let response = request
        .json(event)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, format!("target send failed: {e}")))?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_GATEWAY,
            format!("target returned status {}", response.status()),
        ))
    }
}
