use axum::http::Request;
use axum::middleware::Next;
use std::time::Instant;

pub async fn trace_middleware(req: Request<axum::body::Body>, next: Next) -> axum::response::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    tracing::info!(
        method = %method,
        path = %uri.path(),
        query = ?uri.query(),
        status = status.as_u16(),
        duration_ms = duration.as_millis(),
        "request completed"
    );

    response
}
