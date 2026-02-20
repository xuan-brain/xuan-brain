//! Tauri commands for data folder management
//!
//! This module provides commands for:
//! - Getting current data folder information
//! - Validating new data folder paths
//! - Migrating data to a new location
//! - Restarting the application

use std::path::PathBuf;
use tauri::{AppHandle, State};
use tracing::{error, info};

use crate::service::data_migration_service::DataMigrationService;
use crate::sys::{
    dirs::{
        calculate_data_size, get_data_folder_info, get_default_data_path, save_data_path_config,
        validate_data_folder, DataFolderInfo, DataPathConfig, ValidationResult, AppDirs,
    },
    error::{AppError, Result},
};

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
