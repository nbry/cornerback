use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use std::sync::Arc;

use cornerback_core::models::NewEvent;
use cornerback_core::store::EventStore;

pub fn router(store: Arc<dyn EventStore>) -> Router {
    Router::new()
        .route("/webhook/:id", post(handle_event))
        .route("/webhook/:id/events", get(list_webhook_events))
        .route("/events/:id", get(get_event))
        .with_state(store)
}

async fn handle_event(
    Path(webhook_id): Path<String>,
    State(store): State<Arc<dyn EventStore>>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let event = store
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
    State(store): State<Arc<dyn EventStore>>,
) -> Json<serde_json::Value> {
    let event = store.get_event(id).await.unwrap();

    Json(serde_json::json!(event))
}

// async fn replay_event() {}

async fn list_webhook_events(
    Path(webhook_id): Path<String>,
    State(store): State<Arc<dyn EventStore>>,
) -> Json<serde_json::Value> {
    let events = store
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
