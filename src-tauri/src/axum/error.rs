use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::sys::error::AppError;

pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self.0 {
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::ValidationError { .. } => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            AppError::InvalidInput { .. } => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
            AppError::SurrealDbError { .. } => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = Json(json!({
            "success": false,
            "error": error_type,
            "message": self.0.to_string()
        }));

        (status, body).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(err: AppError) -> Self {
        ApiError(err)
    }
}
