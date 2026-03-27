use std::sync::atomic::Ordering;

use axum::{Json, extract::State, http::StatusCode};

use crate::app::RouteState;

pub async fn start_intercept(State(state): State<RouteState>) -> StatusCode {
    state.intercept.store(true, Ordering::Relaxed);
    tracing::info!("intercept mode enabled");
    StatusCode::NO_CONTENT
}

pub async fn stop_intercept(State(state): State<RouteState>) -> StatusCode {
    state.intercept.store(false, Ordering::Relaxed);
    tracing::info!("intercept mode disabled");
    StatusCode::NO_CONTENT
}

pub async fn intercept_status(State(state): State<RouteState>) -> Json<serde_json::Value> {
    let is_intercept = state.intercept.load(Ordering::Relaxed);
    tracing::debug!("intercept status checked: {}", is_intercept);
    Json(serde_json::json!({
        "intercept": is_intercept
    }))
}
