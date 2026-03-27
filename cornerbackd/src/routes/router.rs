use axum::{
    Router,
    routing::{get, post},
};

use crate::app::RouteState;
use crate::middleware::{trace_middleware, request_id_middleware};
use crate::routes::handlers::{
    get_event, handle_event, intercept_status, list_webhook_events, replay_event, start_intercept,
    stop_intercept,
};

pub fn router(state: RouteState) -> Router {
    Router::new()
        .route("/webhook/:id", post(handle_event))
        .route("/webhook/:id/events", get(list_webhook_events))
        .route("/events/:id", get(get_event))
        .route("/events/:id/replay", post(replay_event))
        .route("/intercept/status", get(intercept_status))
        .route("/intercept/start", post(start_intercept))
        .route("/intercept/stop", post(stop_intercept))
        .layer(axum::middleware::from_fn(trace_middleware))
        .layer(axum::middleware::from_fn(request_id_middleware))
        .with_state(state)
}
