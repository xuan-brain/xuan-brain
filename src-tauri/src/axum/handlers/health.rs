use axum::{http::StatusCode, Json};
use serde_json::json;
use utoipa::ToSchema;

#[derive(serde::Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// Health check endpoint
///
/// Returns the health status of the API service.
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "service": "xuan-brain-api"
        })),
    )
}
