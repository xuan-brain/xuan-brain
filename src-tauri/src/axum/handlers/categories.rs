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
    let categories = CategoryRepository::find_all(&state.db).await.map_err(ApiError)?;

    let result: Vec<CategoryResponse> = categories
        .into_iter()
        .map(|c| CategoryResponse {
            id: c.id.to_string(),
            name: c.name,
            parent_id: c.parent_id.map(|id| id.to_string()),
            sort_order: c.sort_order,
        })
        .collect();

    Ok(Json(result))
}
