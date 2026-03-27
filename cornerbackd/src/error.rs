use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Structured JSON error response
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub error: String,
    pub details: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, error: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(status: StatusCode, error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            status,
            error: error.into(),
            details: Some(details.into()),
        }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, format!("{} not found", resource.into()))
    }

    pub fn bad_request(error: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, error)
    }

    pub fn internal_error(error: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error)
    }

    pub fn bad_gateway(error: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, error)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = if let Some(details) = self.details {
            json!({
                "error": self.error,
                "details": details,
            })
        } else {
            json!({
                "error": self.error,
            })
        };

        (self.status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::internal_error(err.to_string())
    }
}
