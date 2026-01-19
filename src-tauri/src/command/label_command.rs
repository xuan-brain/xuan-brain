use sea_orm::{DatabaseConnection, EntityTrait};
use tauri::State;

use crate::database::entities::{label, prelude::Label};

#[tauri::command]
pub async fn get_all_labels(
    connection: State<'_, DatabaseConnection>,
) -> Result<Vec<label::Model>, String> {
    let labels = Label::find()
        .all(connection.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(labels)
}
