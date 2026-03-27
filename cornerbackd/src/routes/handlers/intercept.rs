use std::sync::atomic::Ordering;

use axum::{Json, extract::State, http::StatusCode};

use crate::app::RouteState;

pub async fn start_intercept(State(state): State<RouteState>) -> StatusCode {
    state.intercept.store(true, Ordering::Relaxed);
    StatusCode::NO_CONTENT
}

pub async fn stop_intercept(State(state): State<RouteState>) -> StatusCode {
    state.intercept.store(false, Ordering::Relaxed);
    StatusCode::NO_CONTENT
}

pub async fn intercept_status(State(state): State<RouteState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "intercept": state.intercept.load(Ordering::Relaxed)
    }))
}
