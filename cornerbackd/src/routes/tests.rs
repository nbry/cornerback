use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::app::RouteState;
use crate::config::{AppConfig, EventStoreConfig, TargetServerConfig};
use crate::routes::router;
use cornerback_core::store::InMemoryEventStore;
use tower::ServiceExt;

#[tokio::test]
async fn test_handle_event() {
    let store = Arc::new(InMemoryEventStore::new());
    let route_state = RouteState {
        store,
        config: Arc::new(AppConfig {
            target_server: TargetServerConfig {
                url: "http://127.0.0.1:65535".to_string(),
                headers: std::collections::HashMap::new(),
            },
            event_store: EventStoreConfig::InMemory,
            send_to_target_by_default: false,
        }),
        http_client: reqwest::Client::new(),
        intercept: Arc::new(AtomicBool::new(true)),
    };
    let app = router(route_state);

    let payload = serde_json::json!({
        "headers": {"x-test": "true"},
        "body": {"foo": "bar"}
    });
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
    assert_eq!(body_json["body"]["foo"], "bar");
}
