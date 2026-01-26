use chrono::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::Serialize;
use tauri::State;

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
    // debug!("Labels: {:?}", labels);

    // For now, return labels without document count
    // TODO: Join with document_labels table to get actual count
    let result: Vec<LabelResponse> = labels
        .into_iter()
        .map(|l| LabelResponse {
            id: l.id,
            name: l.name,
            color: l.color,
            document_count: l.document_count.unwrap_or(0),
        })
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn create_label(
    connection: State<'_, DatabaseConnection>,
    name: String,
    color: String,
) -> Result<LabelResponse, String> {
    let new_label = label::ActiveModel {
        name: Set(name),
        color: Set(color),
        document_count: Set(Some(0)),
        created_at: Set(Local::now().naive_local()),
        updated_at: Set(Local::now().naive_local()),
        ..Default::default()
    };
    let label = new_label
        .insert(connection.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(LabelResponse {
        id: label.id,
        name: label.name,
        color: label.color,
        document_count: label.document_count.unwrap_or(0),
    })
}

#[tauri::command]
pub async fn update_label(
    connection: State<'_, DatabaseConnection>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
) -> Result<LabelResponse, String> {
    let label = Label::find_by_id(id)
        .one(connection.inner())
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Label not found".to_string())?;

    let mut active_model: label::ActiveModel = label.into();

    if let Some(n) = name {
        active_model.name = Set(n);
    }
    if let Some(c) = color {
        active_model.color = Set(c);
    }
    active_model.updated_at = Set(Local::now().naive_local());

    let updated_label = active_model
        .update(connection.inner())
        .await
        .map_err(|e| e.to_string())?;

    Ok(LabelResponse {
        id: updated_label.id,
        name: updated_label.name,
        color: updated_label.color,
        document_count: updated_label.document_count.unwrap_or(0),
    })
}

#[tauri::command]
pub async fn delete_label(
    connection: State<'_, DatabaseConnection>,
    id: i64,
) -> Result<(), String> {
    tracing::info!("Deleting label with id: {}", id);

    let delete_result = Label::delete_by_id(id)
        .exec(connection.inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete label: {}", e);
            e.to_string()
        })?;

    tracing::info!("Delete affected {} rows", delete_result.rows_affected);

    if delete_result.rows_affected == 0 {
        return Err("Label not found".to_string());
    }

    Ok(())
}
