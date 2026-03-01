use axum::{extract::State, Json};
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::repository::LabelRepository;

#[derive(serde::Serialize, ToSchema)]
pub struct LabelResponse {
    pub id: String,
    pub name: String,
    pub color: String,
    pub document_count: i32,
}

/// List all labels
///
/// Returns a list of all labels in the database.
#[utoipa::path(
    get,
    path = "/api/labels",
    tag = "labels",
    responses(
        (status = 200, description = "List of labels", body = Vec<LabelResponse>)
    )
)]
pub async fn list_labels(
    State(state): State<AppState>,
) -> Result<Json<Vec<LabelResponse>>, ApiError> {
    let labels = LabelRepository::find_all(&state.db).await.map_err(ApiError)?;

    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id.to_string(),
            name: l.name,
            color: l.color,
            document_count: l.document_count,
        })
        .collect();

    Ok(Json(result))
}
