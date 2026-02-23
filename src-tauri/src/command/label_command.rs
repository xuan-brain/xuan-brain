use std::sync::Arc;

use serde::Serialize;
use surrealdb_types::RecordIdKey;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::repository::LabelRepository;
use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateLabel, UpdateLabel};
use crate::sys::error::Result;

#[derive(Serialize)]
pub struct LabelResponse {
    pub id: String,
    pub name: String,
    pub color: String,
    pub document_count: i32,
}

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

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_labels(
    db: State<'_, Arc<SurrealClient>>,
) -> Result<Vec<LabelResponse>> {
    info!("Fetching all labels");
    let repo = LabelRepository::new(&db);
    let labels = repo.find_all().await?;

    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
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
    db: State<'_, Arc<SurrealClient>>,
    name: String,
    color: String,
) -> Result<LabelResponse> {
    info!("Creating label '{}' with color '{}'", name, color);
    let repo = LabelRepository::new(&db);
    let label = repo.create(CreateLabel { name: name.clone(), color }).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Created")
        .body(format!("Label '{}' created successfully", name))
        .show();

    info!("Label created successfully");
    Ok(LabelResponse {
        id: label.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
        name: label.name,
        color: label.color,
        document_count: label.document_count,
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_label(
    app: AppHandle,
    db: State<'_, Arc<SurrealClient>>,
    id: String,
    name: Option<String>,
    color: Option<String>,
) -> Result<LabelResponse> {
    info!("Updating label id {}", id);
    let repo = LabelRepository::new(&db);
    let updated_label = repo.update(&id, UpdateLabel { name, color }).await?;

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
        id: updated_label.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
        name: updated_label.name,
        color: updated_label.color,
        document_count: updated_label.document_count,
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn delete_label(
    app: AppHandle,
    db: State<'_, Arc<SurrealClient>>,
    id: String,
) -> Result<()> {
    info!("Deleting label with id: {}", id);
    let repo = LabelRepository::new(&db);
    repo.delete(&id).await?;

    let _ = app.notification()
        .builder()
        .title("Label Deleted")
        .body(format!("Label with id {} deleted successfully", id))
        .show();

    Ok(())
}
