use axum::{
    Json,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Database(String),

    #[error("{0}")]
    Internal(String),
}

impl ApiError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::Database(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::NotFound(_) => "not_found",
            Self::Database(_) => "database_error",
            Self::Internal(_) => "internal_error",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let payload = ErrorResponse {
            error: ErrorBody {
                code: self.error_code(),
                message: self.to_string(),
            },
        };

        (status, Json(payload)).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::database(error.to_string())
    }
}

pub async fn not_found(uri: Uri) -> ApiError {
    ApiError::not_found(format!("No API route exists for `{uri}`"))
}
