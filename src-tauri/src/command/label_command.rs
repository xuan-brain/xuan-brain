use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::{CreateLabel, UpdateLabel};
use crate::repository::LabelRepository;
use crate::sys::error::Result;

#[derive(Serialize)]
pub struct LabelResponse {
    pub id: String,
    pub name: String,
    pub color: String,
    pub document_count: i32,
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_labels(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<LabelResponse>> {
    info!("Fetching all labels");
    let labels = LabelRepository::find_all(&db).await?;

    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id.to_string(),
            name: l.name,
            color: l.color,
            document_count: l.document_count,
        })
        .collect();

    info!("Fetched {} labels", result.len());
    Ok(result)
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn create_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    name: String,
    color: String,
) -> Result<LabelResponse> {
    info!("Creating label '{}' with color '{}'", name, color);
    let label = LabelRepository::create(&db, CreateLabel { name: name.clone(), color }).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Created")
        .body(format!("Label '{}' created successfully", name))
        .show();

    info!("Label created successfully");
    Ok(LabelResponse {
        id: label.id.to_string(),
        name: label.name,
        color: label.color,
        document_count: label.document_count,
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
    name: Option<String>,
    color: Option<String>,
) -> Result<LabelResponse> {
    info!("Updating label id {}", id);

    let id_num = id
        .parse::<i64>()
        .map_err(|_| crate::sys::error::AppError::validation("id", "Invalid id format"))?;

    let updated_label =
        LabelRepository::update(&db, id_num, UpdateLabel { name, color }).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Updated")
        .body(format!(
            "Label '{}' updated successfully",
            updated_label.name
        ))
        .show();

    info!("Label updated successfully");
    Ok(LabelResponse {
        id: updated_label.id.to_string(),
        name: updated_label.name,
        color: updated_label.color,
        document_count: updated_label.document_count,
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn delete_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
) -> Result<()> {
    info!("Deleting label with id: {}", id);

    let id_num = id
        .parse::<i64>()
        .map_err(|_| crate::sys::error::AppError::validation("id", "Invalid id format"))?;

    LabelRepository::delete(&db, id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Deleted")
        .body(format!("Label with id {} deleted successfully", id))
        .show();

    Ok(())
}
