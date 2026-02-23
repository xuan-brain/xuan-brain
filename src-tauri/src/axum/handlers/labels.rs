use axum::{extract::State, Json};
use serde::Serialize;
use surrealdb_types::RecordIdKey;
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::repository::LabelRepository;

/// Convert RecordId to string
fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    format!("{}:{}", id.table, record_id_key_to_string(&id.key))
}

fn record_id_key_to_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        RecordIdKey::Array(_) => "array".to_string(),
        RecordIdKey::Object(_) => "object".to_string(),
        RecordIdKey::Range(_) => "range".to_string(),
    }
}

#[derive(Serialize, ToSchema)]
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
    let repo = LabelRepository::new(&state.db);
    let labels = repo.find_all().await.map_err(ApiError)?;

    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
            name: l.name,
            color: l.color,
            document_count: l.document_count,
        })
        .collect();

    Ok(Json(result))
}
