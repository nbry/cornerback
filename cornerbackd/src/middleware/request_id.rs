use axum::http::Request;
use axum::middleware::Next;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestId(pub Uuid);

impl RequestId {
    pub fn new() -> Self {
        RequestId(Uuid::new_v4())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn request_id_middleware(
    mut req: Request<axum::body::Body>,
    next: Next,
) -> axum::response::Response {
    let request_id = RequestId::new();
    let request_id_str = request_id.0.to_string();

    // Attach request_id to tracing span for correlation
    let span = tracing::Span::current();
    span.record("request_id", &request_id_str);

    // Insert into extensions so handlers can access it
    req.extensions_mut().insert(request_id.clone());

    let mut response = next.run(req).await;

    // Add request_id to response header
    response
        .headers_mut()
        .insert("x-request-id", request_id_str.parse().unwrap());

    response
}
