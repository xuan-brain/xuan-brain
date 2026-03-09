use axum::{extract::State, http::StatusCode, Json};
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::models::CategoryNode;
use crate::repository::CategoryRepository;
use crate::sys::error::AppError;

#[derive(serde::Serialize, ToSchema)]
pub struct CategoryResponse {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

/// Category tree node response with children
/// Note: Recursive type - not fully represented in OpenAPI schema
#[derive(serde::Serialize)]
pub struct CategoryTreeNodeResponse {
    /// Category ID
    pub id: String,
    /// Category name
    pub name: String,
    /// Parent category ID
    pub parent_id: Option<String>,
    /// Sort order
    pub sort_order: i32,
    /// Child categories
    pub children: Vec<CategoryTreeNodeResponse>,
}

impl From<CategoryNode> for CategoryTreeNodeResponse {
    fn from(node: CategoryNode) -> Self {
        Self {
            id: node.id.to_string(),
            name: node.name,
            parent_id: node.parent_id.map(|id| id.to_string()),
            sort_order: node.sort_order,
            children: node
                .children
                .into_iter()
                .map(CategoryTreeNodeResponse::from)
                .collect(),
        }
    }
}

/// Selected category response
#[derive(serde::Serialize, ToSchema)]
pub struct SelectedCategoryResponse {
    /// Selected category ID, None means no selection
    pub selected_category_id: Option<String>,
}

/// Set selected category request
#[derive(serde::Deserialize, ToSchema)]
pub struct SetSelectedCategoryRequest {
    /// Category ID to select, None to deselect
    pub category_id: Option<String>,
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
    let categories = CategoryRepository::find_all(&state.db)
        .await
        .map_err(ApiError)?;

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

/// Get category tree structure
///
/// Returns nested tree structure of all categories.
#[utoipa::path(
    get,
    path = "/api/categories/tree",
    tag = "categories",
    responses(
        (status = 200, description = "Category tree structure (recursive JSON)")
    )
)]
pub async fn get_category_tree(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryTreeNodeResponse>>, ApiError> {
    let tree = CategoryRepository::load_tree(&state.db)
        .await
        .map_err(ApiError)?;

    let result: Vec<CategoryTreeNodeResponse> = tree
        .into_iter()
        .map(CategoryTreeNodeResponse::from)
        .collect();

    Ok(Json(result))
}

/// Get selected category
///
/// Returns the currently selected category ID.
#[utoipa::path(
    get,
    path = "/api/categories/selected",
    tag = "categories",
    responses(
        (status = 200, description = "Selected category", body = SelectedCategoryResponse)
    )
)]
pub async fn get_selected_category(
    State(state): State<AppState>,
) -> Result<Json<SelectedCategoryResponse>, ApiError> {
    let selected_id = state.selected_category.get();

    let response = SelectedCategoryResponse {
        selected_category_id: selected_id.map(|id| id.to_string()),
    };

    Ok(Json(response))
}

/// Set selected category
///
/// Updates the currently selected category.
#[utoipa::path(
    put,
    path = "/api/categories/selected",
    tag = "categories",
    request_body = SetSelectedCategoryRequest,
    responses(
        (status = 200, description = "Category selection updated"),
        (status = 400, description = "Invalid category ID format"),
        (status = 404, description = "Category not found")
    )
)]
pub async fn set_selected_category(
    State(state): State<AppState>,
    Json(payload): Json<SetSelectedCategoryRequest>,
) -> Result<StatusCode, ApiError> {
    let id_value = match payload.category_id {
        Some(id_str) => {
            let id = id_str.parse::<i64>().map_err(|_| {
                ApiError(AppError::validation(
                    "category_id",
                    "Invalid category ID format",
                ))
            })?;
            // Validate category exists (for non-negative IDs)
            if id >= 0 {
                let exists = CategoryRepository::find_by_id(&state.db, id)
                    .await
                    .map_err(ApiError)?;
                if exists.is_none() {
                    return Err(ApiError(AppError::not_found("Category", id_str)));
                }
            }
            Some(id)
        }
        None => None,
    };

    state.selected_category.set(id_value);

    Ok(StatusCode::OK)
}
