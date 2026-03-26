use axum::{Router, routing::{get, post}};

use crate::app::RouteState;
use crate::routes::handlers::{get_event, handle_event, list_webhook_events, replay_event};

pub fn router(state: RouteState) -> Router {
    Router::new()
        .route("/webhook/:id", post(handle_event))
        .route("/webhook/:id/events", get(list_webhook_events))
        .route("/events/:id", get(get_event))
        .route("/events/:id/replay", post(replay_event))
        .with_state(state)
}
