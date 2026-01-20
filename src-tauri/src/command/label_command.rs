use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tauri::State;
use tracing::debug;

use crate::database::entities::{label, prelude::Label};

#[derive(Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub document_count: i64,
}

#[tauri::command]
pub async fn get_all_labels(
    connection: State<'_, DatabaseConnection>,
) -> Result<Vec<LabelResponse>, String> {
    let labels = Label::find()
        .all(connection.inner())
        .await
        .map_err(|e| e.to_string())?;
    debug!("Labels: {:?}", labels);

    // For now, return labels without document count
    // TODO: Join with document_labels table to get actual count
    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id,
            name: l.name,
            color: l.color,
            document_count: l.document_count,
        })
        .collect();

    Ok(result)
}
