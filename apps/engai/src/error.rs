use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use std::fmt;

pub enum AppError {
    Database(sqlx::Error),
    NotFound(String),
    ValidationError(String),
    AiError(String),
    Internal(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::NotFound(msg) => write!(f, "{}", msg),
            AppError::ValidationError(msg) => write!(f, "{}", msg),
            AppError::AiError(msg) => write!(f, "{}", msg),
            AppError::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => f.debug_tuple("Database").field(e).finish(),
            Self::NotFound(msg) => f.debug_tuple("NotFound").field(msg).finish(),
            Self::ValidationError(msg) => f.debug_tuple("ValidationError").field(msg).finish(),
            Self::AiError(msg) => f.debug_tuple("AiError").field(msg).finish(),
            Self::Internal(msg) => f.debug_tuple("Internal").field(msg).finish(),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Database(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

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

pub type ApiResult<T> = std::result::Result<T, ApiError>;
