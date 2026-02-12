use chrono::prelude::*;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use serde::Serialize;
use tauri::State;
use tracing::{info, instrument};

use crate::database::entities::{label, prelude::Label};
use crate::sys::error::{AppError, Result};

#[derive(Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub document_count: i64,
}

#[tauri::command]
#[instrument(skip(connection))]
pub async fn get_all_labels(
    connection: State<'_, DatabaseConnection>,
) -> Result<Vec<LabelResponse>> {
    info!("Fetching all labels");
    let labels = Label::find().all(connection.inner()).await?;

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

    info!("Fetched {} labels", result.len());
    Ok(result)
}

#[tauri::command]
#[instrument(skip(connection))]
pub async fn create_label(
    connection: State<'_, DatabaseConnection>,
    name: String,
    color: String,
) -> Result<LabelResponse> {
    info!("Creating label '{}' with color '{}'", name, color);
    let new_label = label::ActiveModel {
        name: Set(name),
        color: Set(color),
        document_count: Set(Some(0)),
        created_at: Set(Local::now().naive_local()),
        updated_at: Set(Local::now().naive_local()),
        ..Default::default()
    };
    let label = new_label.insert(connection.inner()).await?;

    info!("Label created successfully with id {}", label.id);
    Ok(LabelResponse {
        id: label.id,
        name: label.name,
        color: label.color,
        document_count: label.document_count.unwrap_or(0),
    })
}

#[tauri::command]
#[instrument(skip(connection))]
pub async fn update_label(
    connection: State<'_, DatabaseConnection>,
    id: i64,
    name: Option<String>,
    color: Option<String>,
) -> Result<LabelResponse> {
    info!("Updating label id {}", id);
    let label = Label::find_by_id(id)
        .one(connection.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Label", id.to_string()))?;

    let mut active_model: label::ActiveModel = label.into();

    if let Some(n) = name {
        active_model.name = Set(n);
    }
    if let Some(c) = color {
        active_model.color = Set(c);
    }
    active_model.updated_at = Set(Local::now().naive_local());

    let updated_label = active_model.update(connection.inner()).await?;

    info!("Label updated successfully");
    Ok(LabelResponse {
        id: updated_label.id,
        name: updated_label.name,
        color: updated_label.color,
        document_count: updated_label.document_count.unwrap_or(0),
    })
}

#[tauri::command]
#[instrument(skip(connection))]
pub async fn delete_label(connection: State<'_, DatabaseConnection>, id: i64) -> Result<()> {
    info!("Deleting label with id: {}", id);

    let delete_result = Label::delete_by_id(id).exec(connection.inner()).await?;

    info!("Delete affected {} rows", delete_result.rows_affected);

    if delete_result.rows_affected == 0 {
        return Err(AppError::not_found("Label", id.to_string()));
    }

    Ok(())
}
