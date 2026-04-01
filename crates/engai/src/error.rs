use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use engai_core::error::AppError;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
    pub fn not_found(msg: &str) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }
    pub fn bad_request(msg: &str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }
    pub fn internal(msg: &str) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        match &e {
            AppError::NotFound(msg) => ApiError::not_found(msg),
            AppError::ValidationError(msg) => ApiError::bad_request(msg),
            AppError::AiError(msg) => ApiError::internal(msg),
            AppError::Database(e) => ApiError::internal(&e.to_string()),
            AppError::Internal(msg) => ApiError::internal(msg),
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::internal(&err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": self.message });
        (self.status, axum::Json(body)).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
