//! Mutation operations for papers (create, update, delete)

use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::UpdatePaper;
use crate::repository::{LabelRepository, PaperRepository};
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::parse_id;

/// Migrate abstract field to abstract_text for existing papers
/// This is now a no-op since we're using SQLite
#[tauri::command]
#[instrument(skip(_db))]
pub async fn migrate_abstract_field(
    _db: State<'_, Arc<DatabaseConnection>>,
) -> Result<u64> {
    info!("Migration not needed for SQLite - skipping");
    Ok(0)
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_paper_details(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    payload: UpdatePaperDto,
) -> Result<()> {
    info!("Updating paper details for id {}", payload.id);

    let id_num = parse_id(&payload.id)
        .map_err(|_| AppError::validation("id", "Invalid id format"))?;

    PaperRepository::update(
        &db,
        id_num,
        UpdatePaper {
            title: Some(payload.title.clone()),
            abstract_text: payload.abstract_text,
            doi: payload.doi,
            publication_year: payload.publication_year,
            publication_date: None,
            journal_name: payload.journal_name,
            conference_name: payload.conference_name,
            volume: payload.volume,
            issue: payload.issue,
            pages: payload.pages,
            url: payload.url,
            read_status: payload.read_status,
            notes: payload.notes,
            attachment_path: None,
        },
    )
    .await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Updated")
        .body(format!("Paper '{}' updated successfully", payload.title))
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn delete_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
) -> Result<()> {
    info!("Soft deleting paper with id {}", id);

    let id_num = parse_id(&id)
        .map_err(|_| AppError::validation("id", "Invalid id format"))?;

    PaperRepository::soft_delete(&db, id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Deleted")
        .body("Paper moved to trash")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn restore_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
) -> Result<()> {
    info!("Restoring paper with id {}", id);

    let id_num = parse_id(&id)
        .map_err(|_| AppError::validation("id", "Invalid id format"))?;

    PaperRepository::restore(&db, id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Restored")
        .body("Paper restored from trash successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn permanently_delete_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
) -> Result<()> {
    info!("Permanently deleting paper with id {}", id);

    let id_num = parse_id(&id)
        .map_err(|_| AppError::validation("id", "Invalid id format"))?;

    PaperRepository::delete(&db, id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Deleted Permanently")
        .body("Paper permanently deleted successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_paper_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: String,
    category_id: Option<String>,
) -> Result<()> {
    info!("Updating category for paper {}: {:?}", paper_id, category_id);

    let paper_id_num = parse_id(&paper_id)
        .map_err(|_| AppError::validation("paper_id", "Invalid id format"))?;

    let category_id_num = if let Some(cat_id) = category_id {
        Some(parse_id(&cat_id).map_err(|_| {
            AppError::validation("category_id", "Invalid id format")
        })?)
    } else {
        None
    };

    PaperRepository::set_category(&db, paper_id_num, category_id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Category Updated")
        .body("Paper category updated successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn add_paper_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: String,
    label_id: String,
) -> Result<()> {
    info!("Adding label {} to paper {}", label_id, paper_id);

    let paper_id_num = parse_id(&paper_id)
        .map_err(|_| AppError::validation("paper_id", "Invalid id format"))?;
    let label_id_num = parse_id(&label_id)
        .map_err(|_| AppError::validation("label_id", "Invalid id format"))?;

    LabelRepository::add_to_paper(&db, paper_id_num, label_id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Added")
        .body("Label added to paper successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn remove_paper_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: String,
    label_id: String,
) -> Result<()> {
    info!("Removing label {} from paper {}", label_id, paper_id);

    let paper_id_num = parse_id(&paper_id)
        .map_err(|_| AppError::validation("paper_id", "Invalid id format"))?;
    let label_id_num = parse_id(&label_id)
        .map_err(|_| AppError::validation("label_id", "Invalid id format"))?;

    LabelRepository::remove_from_paper(&db, paper_id_num, label_id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Removed")
        .body("Label removed from paper successfully")
        .show();

    Ok(())
}
