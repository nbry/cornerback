use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};

use crate::app::RouteState;
use cornerback_core::models::NewEvent;

pub fn router(state: RouteState) -> Router {
    Router::new()
        .route("/webhook/:id", post(handle_event))
        .route("/webhook/:id/events", get(list_webhook_events))
        .route("/events/:id", get(get_event))
        .with_state(state)
}

async fn handle_event(
    Path(webhook_id): Path<String>,
    State(state): State<RouteState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event = state
        .store
        .insert_event(NewEvent {
            webhook_id,
            headers: serde_json::json!({}),
            body: payload,
        })
        .await
        .unwrap();

    Json(serde_json::json!({ "id": event.id }))
}

async fn get_event(
    Path(id): Path<uuid::Uuid>,
    State(state): State<RouteState>,
) -> Json<serde_json::Value> {
    let event = state.store.get_event(id).await.unwrap();

    Json(serde_json::json!(event))
}

// async fn replay_event() {}

async fn list_webhook_events(
    Path(webhook_id): Path<String>,
    State(state): State<RouteState>,
) -> Json<serde_json::Value> {
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
        .unwrap();

    Json(serde_json::json!(events))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use cornerback_core::store::InMemoryEventStore;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handle_event() {
        let store = Arc::new(InMemoryEventStore::new());
        let route_state = RouteState { store };
        let app = router(route_state);

        let payload = serde_json::json!({ "foo": "bar" });
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/webhook/test")
                    .header("Content-Type", "application/json")
                    .body(axum::body::Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert!(body_json.get("id").is_some());
    }
}
