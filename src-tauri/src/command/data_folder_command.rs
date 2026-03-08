//! Tauri commands for data folder management
//!
//! This module provides commands for:
//! - Getting current data folder information
//! - Validating new data folder paths
//! - Migrating data to a new location
//! - Restarting the application
//! - Clearing all data (dev mode only)

use std::path::PathBuf;
use std::sync::Arc;

use sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tauri::{AppHandle, State};
use tracing::{error, info};

use crate::database::entities::{
    attachment, label, paper, paper_author, paper_category, paper_keyword, paper_label,
};
use crate::service::data_migration_service::DataMigrationService;
use crate::sys::{
    dirs::{
        calculate_data_size, get_data_folder_info, get_default_data_path, save_data_path_config,
        validate_data_folder, DataFolderInfo, DataPathConfig, ValidationResult, AppDirs,
    },
    error::{AppError, Result},
};

/// Result of clear all data operation
#[derive(Debug, Serialize, Clone)]
pub struct ClearDataResult {
    pub papers_deleted: u64,
    pub labels_deleted: u64,
    pub files_deleted: u64,
    pub errors: Vec<String>,
}

/// Get current data folder information
#[tauri::command]
pub async fn get_data_folder_info_command(app_dirs: State<'_, AppDirs>) -> Result<DataFolderInfo> {
    info!("Getting data folder information");
    get_data_folder_info(&app_dirs)
}

/// Get the default system data folder path
#[tauri::command]
pub async fn get_default_data_folder() -> Result<String> {
    info!("Getting default data folder path");
    get_default_data_path()
}

/// Validate a potential new data folder path
#[tauri::command]
pub async fn validate_data_folder_command(
    path: String,
    app_dirs: State<'_, AppDirs>,
) -> Result<ValidationResult> {
    info!("Validating data folder path: {}", path);

    // Calculate required space (current data size + 10% buffer)
    let current_size = calculate_data_size(&app_dirs).unwrap_or(0);
    let required_space = current_size + (current_size / 10);

    validate_data_folder(&path, required_space)
}

/// Migrate data to a new folder
#[tauri::command]
pub async fn migrate_data_folder_command(
    app: AppHandle,
    new_path: String,
    app_dirs: State<'_, AppDirs>,
) -> Result<()> {
    info!("Starting data migration to: {}", new_path);

    // Get current base directory (parent of XuanBrain folder)
    // app_dirs.data is {base}/XuanBrain/data, so we need parent twice to get {base}
    let current_base = PathBuf::from(&app_dirs.data)
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| AppError::migration_error("migrate", "Invalid current data path"))?
        .to_path_buf();

    let new_base = PathBuf::from(&new_path);

    // Validate the new path
    let current_size = calculate_data_size(&app_dirs).unwrap_or(0);
    let required_space = current_size + (current_size / 10);
    let validation = validate_data_folder(&new_path, required_space)?;

    if !validation.valid {
        return Err(AppError::migration_error(
            "validate",
            validation.error.unwrap_or_else(|| "Invalid path".to_string()),
        ));
    }

    // Create migration service
    let migration_service = DataMigrationService::new(current_base, new_base);

    // Execute migration
    match migration_service.migrate(&app).await {
        Ok(_) => {
            info!("Data migration completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Data migration failed: {}", e);

            // Attempt rollback
            if let Err(rollback_err) = migration_service.rollback(&app) {
                error!("Rollback also failed: {}", rollback_err);
            }

            Err(e)
        }
    }
}

/// Revert to default data folder
#[tauri::command]
pub async fn revert_to_default_data_folder_command(
    app: AppHandle,
    app_dirs: State<'_, AppDirs>,
) -> Result<()> {
    info!("Reverting to default data folder");

    // Get default data path - get_default_data_path returns {base}/XuanBrain, so parent gives {base}
    let default_base = PathBuf::from(get_default_data_path()?)
        .parent()
        .ok_or_else(|| AppError::migration_error("revert", "Invalid default path"))?
        .to_path_buf();

    // Get current base directory (parent of XuanBrain folder)
    // app_dirs.data is {base}/XuanBrain/data, so we need parent twice to get {base}
    let current_base = PathBuf::from(&app_dirs.data)
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| AppError::migration_error("revert", "Invalid current data path"))?
        .to_path_buf();

    // Create migration service
    let migration_service = DataMigrationService::new(current_base, default_base);

    // Execute migration
    match migration_service.migrate(&app).await {
        Ok(_) => {
            // Clear custom path in config
            let config = DataPathConfig {
                custom_data_path: None,
                version: 1,
                pending_cleanup_path: None,
            };
            save_data_path_config(&config)?;

            info!("Revert to default completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("Revert to default failed: {}", e);

            // Attempt rollback
            if let Err(rollback_err) = migration_service.rollback(&app) {
                error!("Rollback also failed: {}", rollback_err);
            }

            Err(e)
        }
    }
}

/// Restart the application
#[tauri::command]
pub async fn restart_app(app: AppHandle) -> Result<()> {
    info!("Restarting application...");

    // Use tauri-plugin-process to restart
    app.restart();
}

/// Clear all data from the database (dev mode only)
///
/// This command deletes:
/// - All papers (including soft-deleted)
/// - All labels
/// - All attachments
/// - All files in the files directory
/// - All relations (paper-author, paper-label, paper-keyword, paper-category)
///
/// Category data is preserved.
#[tauri::command]
pub async fn clear_all_data_command(
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<ClearDataResult> {
    info!("Starting clear all data operation (dev mode)");

    let mut result = ClearDataResult {
        papers_deleted: 0,
        labels_deleted: 0,
        files_deleted: 0,
        errors: Vec::new(),
    };

    // Delete in correct order to handle foreign key constraints
    // 1. Delete all paper_label relations
    match paper_label::Entity::delete_many().exec(db.as_ref()).await {
        Ok(_) => info!("Deleted all paper_label relations"),
        Err(e) => result.errors.push(format!("Failed to delete paper_labels: {}", e)),
    }

    // 2. Delete all paper_author relations
    match paper_author::Entity::delete_many().exec(db.as_ref()).await {
        Ok(_) => info!("Deleted all paper_author relations"),
        Err(e) => result.errors.push(format!("Failed to delete paper_authors: {}", e)),
    }

    // 3. Delete all paper_keyword relations
    match paper_keyword::Entity::delete_many().exec(db.as_ref()).await {
        Ok(_) => info!("Deleted all paper_keyword relations"),
        Err(e) => result.errors.push(format!("Failed to delete paper_keywords: {}", e)),
    }

    // 4. Delete all paper_category relations (keep categories)
    match paper_category::Entity::delete_many().exec(db.as_ref()).await {
        Ok(_) => info!("Deleted all paper_category relations"),
        Err(e) => result.errors.push(format!("Failed to delete paper_categories: {}", e)),
    }

    // 5. Delete all attachments
    match attachment::Entity::delete_many().exec(db.as_ref()).await {
        Ok(r) => {
            let deleted = r.rows_affected;
            info!("Deleted {} attachments", deleted);
        }
        Err(e) => result.errors.push(format!("Failed to delete attachments: {}", e)),
    }

    // 6. Delete all papers (including soft-deleted)
    match paper::Entity::delete_many().exec(db.as_ref()).await {
        Ok(r) => {
            result.papers_deleted = r.rows_affected;
            info!("Deleted {} papers", result.papers_deleted);
        }
        Err(e) => result.errors.push(format!("Failed to delete papers: {}", e)),
    }

    // 7. Delete all labels
    match label::Entity::delete_many().exec(db.as_ref()).await {
        Ok(r) => {
            result.labels_deleted = r.rows_affected;
            info!("Deleted {} labels", result.labels_deleted);
        }
        Err(e) => result.errors.push(format!("Failed to delete labels: {}", e)),
    }

    // 8. Clear files directory
    let files_path = PathBuf::from(&app_dirs.files);
    if files_path.exists() {
        match clear_directory_contents(&files_path) {
            Ok(count) => {
                result.files_deleted = count;
                info!("Deleted {} items from files directory", count);
            }
            Err(e) => {
                result.errors.push(format!("Failed to clear files directory: {}", e));
                error!("Failed to clear files directory: {}", e);
            }
        }
    }

    info!("Clear all data operation completed: {:?}", result);
    Ok(result)
}

/// Clear all contents of a directory without removing the directory itself
fn clear_directory_contents(dir: &PathBuf) -> std::result::Result<u64, String> {
    let mut count = 0u64;

    if !dir.is_dir() {
        return Err(format!("Path is not a directory: {:?}", dir));
    }

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| e.to_string())?;
            count += 1;
        } else {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}
