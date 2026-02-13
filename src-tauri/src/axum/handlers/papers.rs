use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::EntityTrait;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::database::entities::papers;

pub async fn list_papers(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let db = &*state.db;
    let papers = papers::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    let result: Vec<serde_json::Value> = papers
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "abstract": p.r#abstract,
                "doi": p.doi,
                "publication_year": p.publication_year,
                "journal_name": p.journal_name,
                "url": p.url,
                "read_status": p.read_status,
                "created_at": p.created_at,
                "updated_at": p.updated_at
            })
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_paper(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = &*state.db;
    let paper = papers::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    match paper {
        Some(p) => Ok(Json(serde_json::json!({
            "id": p.id,
            "title": p.title,
            "abstract": p.r#abstract,
            "doi": p.doi,
            "publication_year": p.publication_year,
            "journal_name": p.journal_name,
            "url": p.url,
            "notes": p.notes,
            "read_status": p.read_status,
            "created_at": p.created_at,
            "updated_at": p.updated_at
        }))),
        None => Err(ApiError(crate::sys::error::AppError::NotFound {
            resource_type: "Paper".to_string(),
            resource_id: id.to_string(),
        })),
    }
}
