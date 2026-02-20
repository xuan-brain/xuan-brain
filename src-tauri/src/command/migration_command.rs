//! Migration commands for SQLite to SurrealDB migration
//!
//! These commands provide functionality to migrate data from SQLite to SurrealDB.

use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::database::entities::papers;
use crate::service::surreal_migration_service::{MigrationReport, SurrealMigrator};
use crate::surreal::connection::SurrealClient;
use crate::sys::error::Result;

/// Migration status response
#[derive(Serialize, Deserialize)]
pub struct MigrationStatus {
    pub can_migrate: bool,
    pub sqlite_papers_count: i64,
    pub surreal_papers_count: i64,
    pub message: String,
}

/// Check migration status
#[tauri::command]
#[instrument(skip(sqlite, surreal))]
pub async fn get_migration_status(
    sqlite: State<'_, Arc<DatabaseConnection>>,
    surreal: State<'_, Arc<SurrealClient>>,
) -> Result<MigrationStatus> {
    info!("Checking migration status");

    // Count papers in SQLite
    let sqlite_count: u64 = papers::Entity::find()
        .filter(papers::Column::DeletedAt.is_null())
        .count(sqlite.inner().as_ref())
        .await
        .unwrap_or(0);

    // Count papers in SurrealDB - use count() function which returns a single number
    let surreal_count: Option<i64> = surreal
        .query("SELECT count() FROM paper WHERE deleted_at IS NONE GROUP ALL")
        .await
        .map_err(|e| crate::sys::error::AppError::surrealdb_error("count_papers", e.to_string()))?
        .take((0, "count"))
        .map_err(|e| crate::sys::error::AppError::surrealdb_error("count_papers_result", e.to_string()))?;

    let surreal_count = surreal_count.unwrap_or(0);

    let can_migrate = sqlite_count > 0;
    let needs_migration = sqlite_count as i64 > surreal_count;

    let message = if !can_migrate {
        "No data to migrate".to_string()
    } else if needs_migration {
        format!("{} papers in SQLite, {} in SurrealDB. Migration recommended.", sqlite_count, surreal_count)
    } else {
        "Migration already completed".to_string()
    };

    Ok(MigrationStatus {
        can_migrate,
        sqlite_papers_count: sqlite_count as i64,
        surreal_papers_count: surreal_count,
        message,
    })
}

/// Run the full migration from SQLite to SurrealDB
#[tauri::command]
#[instrument(skip(app, sqlite, surreal))]
pub async fn run_migration(
    app: AppHandle,
    sqlite: State<'_, Arc<DatabaseConnection>>,
    surreal: State<'_, Arc<SurrealClient>>,
) -> Result<MigrationReport> {
    info!("Starting SQLite to SurrealDB migration");

    let migrator = SurrealMigrator::new(sqlite.inner().clone(), surreal.inner().clone());
    let report = migrator.migrate_all().await?;

    info!(
        "Migration completed: {} papers, {} authors, {} categories, {} labels",
        report.papers_migrated,
        report.authors_migrated,
        report.categories_migrated,
        report.labels_migrated
    );

    // Send notification
    let _ = app
        .notification()
        .builder()
        .title("Migration Completed")
        .body(format!(
            "Migrated {} papers to SurrealDB",
            report.papers_migrated
        ))
        .show();

    Ok(report)
}

/// Verify migration by comparing counts
#[tauri::command]
#[instrument(skip(sqlite, surreal))]
pub async fn verify_migration(
    sqlite: State<'_, Arc<DatabaseConnection>>,
    surreal: State<'_, Arc<SurrealClient>>,
) -> Result<MigrationReport> {
    info!("Verifying migration");

    let migrator = SurrealMigrator::new(sqlite.inner().clone(), surreal.inner().clone());
    migrator.verify_counts().await
}
