use axum::{extract::State, Json};
use sea_orm::EntityTrait;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::database::entities::category;

pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let db = &*state.db;
    let categories = category::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    let result: Vec<serde_json::Value> = categories
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "parent_id": c.parent_id,
                "sort_order": c.sort_order
            })
        })
        .collect();

    Ok(Json(result))
}
