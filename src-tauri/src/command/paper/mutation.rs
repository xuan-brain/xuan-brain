//! Mutation operations for papers (create, update, delete)

use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::repository::{LabelRepository, PaperRepository};
use crate::surreal::connection::SurrealClient;
use crate::surreal::models::UpdatePaper;
use crate::sys::error::Result;

use super::dtos::*;

/// Migrate abstract field to abstract_text for existing papers
/// Call this once after upgrading from versions that used `abstract` field
#[tauri::command]
#[instrument(skip(db))]
pub async fn migrate_abstract_field(
    db: State<'_, Arc<SurrealClient>>,
) -> Result<u64> {
    info!("Starting abstract field migration");
    let repo = PaperRepository::new(&db);
    let count = repo.migrate_abstract_field().await?;
    info!("Migration completed: {} papers updated", count);
    Ok(count)
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_paper_details(
    app: AppHandle,
    db: State<'_, Arc<SurrealClient>>,
    payload: UpdatePaperDto,
) -> Result<()> {
    info!("Updating paper details for id {}", payload.id);
    let repo = PaperRepository::new(&db);

    repo.update(&payload.id, UpdatePaper {
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
        attachments: None,
    }).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    id: String,
) -> Result<()> {
    info!("Soft deleting paper with id {}", id);
    let repo = PaperRepository::new(&db);

    repo.soft_delete(&id).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    id: String,
) -> Result<()> {
    info!("Restoring paper with id {}", id);
    let repo = PaperRepository::new(&db);

    repo.restore(&id).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    id: String,
) -> Result<()> {
    info!("Permanently deleting paper with id {}", id);
    let repo = PaperRepository::new(&db);

    repo.delete(&id).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    paper_id: String,
    category_id: Option<String>,
) -> Result<()> {
    info!("Updating category for paper {}: {:?}", paper_id, category_id);
    let repo = PaperRepository::new(&db);

    repo.set_category(&paper_id, category_id).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    paper_id: String,
    label_id: String,
) -> Result<()> {
    info!("Adding label {} to paper {}", label_id, paper_id);
    let repo = LabelRepository::new(&db);

    repo.add_to_paper(&paper_id, &label_id).await?;

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
    db: State<'_, Arc<SurrealClient>>,
    paper_id: String,
    label_id: String,
) -> Result<()> {
    info!("Removing label {} from paper {}", label_id, paper_id);
    let repo = LabelRepository::new(&db);

    repo.remove_from_paper(&paper_id, &label_id).await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Removed")
        .body("Label removed from paper successfully")
        .show();

    Ok(())
}
