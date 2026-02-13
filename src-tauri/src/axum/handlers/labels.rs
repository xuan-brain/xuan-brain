use axum::{extract::State, Json};
use sea_orm::EntityTrait;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::database::entities::label;

pub async fn list_labels(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let db = &*state.db;
    let labels = label::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    let result: Vec<serde_json::Value> = labels
        .into_iter()
        .map(|l| {
            serde_json::json!({
                "id": l.id,
                "name": l.name,
                "color": l.color
            })
        })
        .collect();

    Ok(Json(result))
}
