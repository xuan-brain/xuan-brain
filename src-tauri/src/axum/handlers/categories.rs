use axum::{extract::State, Json};
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::repository::CategoryRepository;

#[derive(serde::Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

/// RecordId to string helper
fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    use surrealdb_types::RecordIdKey;
    format!("{}:{}", id.table, match &id.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => "unknown".to_string(),
    })
}

/// List all categories
///
/// Returns a list of all categories in the database.
#[utoipa::path(
    get,
    path = "/api/categories",
    tag = "categories",
    responses(
        (status = 200, description = "List of categories", body = Vec<CategoryResponse>)
    )
)]
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryResponse>>, ApiError> {
    let repo = CategoryRepository::new(&state.db);
    let categories = repo.find_all().await.map_err(ApiError)?;

    let result: Vec<CategoryResponse> = categories
        .into_iter()
        .map(|c| CategoryResponse {
            id: c.id.as_ref().map(record_id_to_string).unwrap_or_default(),
            name: c.name,
            parent_id: c.parent.map(|rid| record_id_to_string(&rid)),
            sort_order: c.sort_order,
        })
        .collect();

    Ok(Json(result))
}
