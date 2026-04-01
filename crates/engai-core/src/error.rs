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
