//! Data migration service for moving application data between folders
//!
//! This module provides functionality to migrate all application data
//! (database, files, cache, config, logs) from one location to another.

use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

use crate::sys::{
    consts::APP_FOLDER,
    dirs::{save_data_path_config, DataPathConfig, MigrationPhase, MigrationStatus},
    error::{AppError, Result},
};

/// Data migration service
pub struct DataMigrationService {
    /// Source base directory (parent of XuanBrain folder)
    source_base: PathBuf,
    /// Destination base directory (parent of XuanBrain folder)
    dest_base: PathBuf,
}

impl DataMigrationService {
    /// Create a new migration service
    pub fn new(source_base: PathBuf, dest_base: PathBuf) -> Self {
        Self {
            source_base,
            dest_base,
        }
    }

    /// Get the actual XuanBrain directory from a base path
    /// If the path already ends with APP_FOLDER, return it directly
    /// Otherwise, append APP_FOLDER
    fn get_xuanbrain_dir(base: &Path) -> PathBuf {
        if base.file_name()
            .map(|name| name.to_string_lossy() == APP_FOLDER)
            .unwrap_or(false)
        {
            base.to_path_buf()
        } else {
            base.join(APP_FOLDER)
        }
    }

    /// Get the parent directory (for saving to config)
    /// If the path ends with APP_FOLDER, return its parent
    /// Otherwise, return the path as-is
    fn get_parent_dir(base: &Path) -> PathBuf {
        if base.file_name()
            .map(|name| name.to_string_lossy() == APP_FOLDER)
            .unwrap_or(false)
        {
            base.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| base.to_path_buf())
        } else {
            base.to_path_buf()
        }
    }

    /// Execute the migration process
    pub async fn migrate(&self, app_handle: &AppHandle) -> Result<()> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base);
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base);

        info!(
            "Starting data migration from {:?} to {:?}",
            source_dir, dest_dir
        );

        // Emit initial status
        self.emit_status(app_handle, MigrationPhase::Preparing, 0, 100, None, None)?;

        // Prepare for migration
        self.prepare()?;

        // Count total files for progress tracking
        let total_files = self.count_files()?;
        let mut processed_files: u32 = 0;

        // Copy database
        self.emit_status(
            app_handle,
            MigrationPhase::CopyingDatabase,
            processed_files,
            total_files,
            None,
            None,
        )?;
        processed_files += self.copy_database(app_handle, total_files, processed_files).await?;

        // Copy config files
        self.emit_status(
            app_handle,
            MigrationPhase::CopyingConfig,
            processed_files,
            total_files,
            None,
            None,
        )?;
        processed_files += self.copy_config(app_handle, total_files, processed_files)?;

        // Copy files (PDF attachments)
        self.emit_status(
            app_handle,
            MigrationPhase::CopyingFiles,
            processed_files,
            total_files,
            None,
            None,
        )?;
        processed_files += self.copy_files(app_handle, total_files, processed_files)?;

        // Copy cache
        self.emit_status(
            app_handle,
            MigrationPhase::CopyingCache,
            processed_files,
            total_files,
            None,
            None,
        )?;
        processed_files += self.copy_cache(app_handle, total_files, processed_files)?;

        // Copy logs
        self.emit_status(
            app_handle,
            MigrationPhase::CopyingLogs,
            processed_files,
            total_files,
            None,
            None,
        )?;
        let _ = self.copy_logs(app_handle, total_files, processed_files)?;

        // Verify migration
        self.emit_status(
            app_handle,
            MigrationPhase::Verifying,
            total_files,
            total_files,
            None,
            None,
        )?;
        self.verify()?;

        // Update configuration with pending cleanup path
        // Save the path without APP_FOLDER suffix (the actual parent directory)
        // If the path already ends with APP_FOLDER, save its parent instead
        let config_path = if self.dest_base.file_name()
            .map(|name| name.to_string_lossy() == APP_FOLDER)
            .unwrap_or(false)
        {
            self.dest_base.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| self.dest_base.to_string_lossy().to_string())
        } else {
            self.dest_base.to_string_lossy().to_string()
        };

        // Record source path for cleanup on next startup
        let source_cleanup_path = Self::get_xuanbrain_dir(&self.source_base)
            .to_string_lossy()
            .to_string();

        let config = DataPathConfig {
            custom_data_path: Some(config_path),
            version: 1,
            pending_cleanup_path: Some(source_cleanup_path),
        };
        save_data_path_config(&config)?;

        // Emit completion status
        self.emit_status(
            app_handle,
            MigrationPhase::Completed,
            total_files,
            total_files,
            None,
            None,
        )?;

        info!("Data migration completed successfully");
        Ok(())
    }

    /// Prepare for migration
    fn prepare(&self) -> Result<()> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base);
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base);

        // Verify source exists
        if !source_dir.exists() {
            return Err(AppError::migration_error(
                "prepare",
                format!("Source directory does not exist: {:?}", source_dir),
            ));
        }

        // Create destination directory
        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("prepare", format!("Failed to create destination directory: {}", e))
        })?;

        info!("Migration preparation completed");
        Ok(())
    }

    /// Count total files to migrate for progress tracking
    fn count_files(&self) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base);
        let mut count: u32 = 0;

        let subdirs = ["data", "files", "cache", "config", "logs"];
        for subdir in subdirs {
            let dir_path = source_dir.join(subdir);
            if dir_path.exists() {
                count += count_files_in_dir(&dir_path)?;
            }
        }

        Ok(count.max(1)) // At least 1 to avoid division by zero
    }

    /// Copy database files
    async fn copy_database(
        &self,
        app_handle: &AppHandle,
        total_files: u32,
        mut processed_files: u32,
    ) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base).join("data");
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base).join("data");

        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("copy_database", format!("Failed to create data directory: {}", e))
        })?;

        let mut copied: u32 = 0;

        // Note: Database connections will be closed when the app restarts
        // This is handled by the app lifecycle

        if source_dir.exists() {
            for entry in fs::read_dir(&source_dir).map_err(|e| {
                AppError::migration_error("copy_database", format!("Failed to read source directory: {}", e))
            })? {
                let entry = entry.map_err(|e| {
                    AppError::migration_error("copy_database", format!("Failed to read entry: {}", e))
                })?;

                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy().to_string();
                let dest_path = dest_dir.join(&file_name);

                // For SQLite database files, ensure proper handling
                if file_name_str.ends_with(".sqlite") || file_name_str.ends_with(".db") {
                    // Copy with progress update
                    fs::copy(entry.path(), &dest_path).map_err(|e| {
                        AppError::migration_error(
                            "copy_database",
                            format!("Failed to copy database file {}: {}", file_name_str, e),
                        )
                    })?;

                    copied += 1;
                    processed_files += 1;

                    self.emit_status(
                        app_handle,
                        MigrationPhase::CopyingDatabase,
                        processed_files,
                        total_files,
                        Some(file_name_str),
                        None,
                    )?;
                } else {
                    // Copy other files in data directory
                    if entry.path().is_file() {
                        fs::copy(entry.path(), &dest_path).map_err(|e| {
                            AppError::migration_error(
                                "copy_database",
                                format!("Failed to copy file {}: {}", file_name_str, e),
                            )
                        })?;
                        copied += 1;
                        processed_files += 1;
                    }
                }
            }
        }

        info!("Copied {} database files", copied);
        Ok(copied)
    }

    /// Copy config files
    fn copy_config(
        &self,
        app_handle: &AppHandle,
        total_files: u32,
        processed_files: u32,
    ) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base).join("config");
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base).join("config");

        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("copy_config", format!("Failed to create config directory: {}", e))
        })?;

        let copied = copy_directory_with_progress(
            &source_dir,
            &dest_dir,
            app_handle,
            MigrationPhase::CopyingConfig,
            total_files,
            processed_files,
        )?;

        info!("Copied {} config files", copied);
        Ok(copied)
    }

    /// Copy files (PDF attachments)
    fn copy_files(
        &self,
        app_handle: &AppHandle,
        total_files: u32,
        processed_files: u32,
    ) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base).join("files");
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base).join("files");

        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("copy_files", format!("Failed to create files directory: {}", e))
        })?;

        let copied = copy_directory_with_progress(
            &source_dir,
            &dest_dir,
            app_handle,
            MigrationPhase::CopyingFiles,
            total_files,
            processed_files,
        )?;

        info!("Copied {} user files", copied);
        Ok(copied)
    }

    /// Copy cache files
    fn copy_cache(
        &self,
        app_handle: &AppHandle,
        total_files: u32,
        processed_files: u32,
    ) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base).join("cache");
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base).join("cache");

        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("copy_cache", format!("Failed to create cache directory: {}", e))
        })?;

        let copied = copy_directory_with_progress(
            &source_dir,
            &dest_dir,
            app_handle,
            MigrationPhase::CopyingCache,
            total_files,
            processed_files,
        )?;

        info!("Copied {} cache files", copied);
        Ok(copied)
    }

    /// Copy log files
    fn copy_logs(
        &self,
        app_handle: &AppHandle,
        total_files: u32,
        processed_files: u32,
    ) -> Result<u32> {
        let source_dir = Self::get_xuanbrain_dir(&self.source_base).join("logs");
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base).join("logs");

        fs::create_dir_all(&dest_dir).map_err(|e| {
            AppError::migration_error("copy_logs", format!("Failed to create logs directory: {}", e))
        })?;

        let copied = copy_directory_with_progress(
            &source_dir,
            &dest_dir,
            app_handle,
            MigrationPhase::CopyingLogs,
            total_files,
            processed_files,
        )?;

        info!("Copied {} log files", copied);
        Ok(copied)
    }

    /// Verify migration completed successfully
    fn verify(&self) -> Result<()> {
        let dest_dir = Self::get_xuanbrain_dir(&self.dest_base);

        // Verify destination directories exist
        let subdirs = ["data", "files", "cache", "config", "logs"];
        for subdir in subdirs {
            let dest_subdir = dest_dir.join(subdir);
            if !dest_subdir.exists() {
                return Err(AppError::migration_error(
                    "verify",
                    format!("Destination directory missing: {:?}", dest_subdir),
                ));
            }
        }

        // Verify database file exists
        let db_path = dest_dir.join("data").join("xuan_brain.sqlite");
        if !db_path.exists() {
            warn!("Database file not found at {:?}, may be a new installation", db_path);
        }

        info!("Migration verification completed successfully");
        Ok(())
    }

    /// Emit migration status to frontend
    fn emit_status(
        &self,
        app_handle: &AppHandle,
        phase: MigrationPhase,
        processed_files: u32,
        total_files: u32,
        current_file: Option<String>,
        error: Option<String>,
    ) -> Result<()> {
        let status = MigrationStatus {
            phase,
            current_file,
            total_files,
            processed_files,
            error,
        };

        app_handle
            .emit("data-migration-progress", &status)
            .map_err(|e| {
                AppError::migration_error("emit_status", format!("Failed to emit status: {}", e))
            })?;

        Ok(())
    }

    /// Rollback migration in case of failure
    pub fn rollback(&self, app_handle: &AppHandle) -> Result<()> {
        info!("Starting migration rollback...");

        self.emit_status(
            app_handle,
            MigrationPhase::RollingBack,
            0,
            100,
            None,
            None,
        )?;

        let dest_dir = self.dest_base.join(APP_FOLDER);

        // Remove partially copied destination directory
        if dest_dir.exists() {
            fs::remove_dir_all(&dest_dir).map_err(|e| {
                AppError::migration_error("rollback", format!("Failed to remove destination directory: {}", e))
            })?;
        }

        // Reset configuration to source
        let config = DataPathConfig {
            custom_data_path: if self.source_base
                == dirs::data_dir()
                    .unwrap_or_default()
                    .parent()
                    .unwrap_or(&PathBuf::from(""))
            {
                None
            } else {
                Some(self.source_base.to_string_lossy().to_string())
            },
            version: 1,
            pending_cleanup_path: None,
        };
        save_data_path_config(&config)?;

        info!("Rollback completed");
        Ok(())
    }
}

/// Count files in a directory recursively
fn count_files_in_dir(path: &PathBuf) -> Result<u32> {
    let mut count: u32 = 0;

    if !path.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(path).map_err(|e| {
        AppError::file_system(path.display().to_string(), format!("Failed to read directory: {}", e))
    })? {
        let entry = entry.map_err(|e| {
            AppError::file_system(path.display().to_string(), format!("Failed to read entry: {}", e))
        })?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            count += count_files_in_dir(&entry_path)?;
        } else {
            count += 1;
        }
    }

    Ok(count)
}

/// Copy a directory recursively with progress updates
fn copy_directory_with_progress(
    source: &PathBuf,
    dest: &PathBuf,
    app_handle: &AppHandle,
    phase: MigrationPhase,
    total_files: u32,
    mut processed_files: u32,
) -> Result<u32> {
    if !source.exists() {
        return Ok(0);
    }

    let mut copied: u32 = 0;

    fn copy_dir_recursive(
        src: &PathBuf,
        dst: &PathBuf,
        app_handle: &AppHandle,
        phase: &MigrationPhase,
        total_files: u32,
        processed_files: &mut u32,
        copied: &mut u32,
    ) -> Result<()> {
        fs::create_dir_all(dst).map_err(|e| {
            AppError::migration_error("copy_dir", format!("Failed to create directory: {}", e))
        })?;

        for entry in fs::read_dir(src).map_err(|e| {
            AppError::migration_error("copy_dir", format!("Failed to read directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                AppError::migration_error("copy_dir", format!("Failed to read entry: {}", e))
            })?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dst.join(&file_name);

            if entry_path.is_dir() {
                copy_dir_recursive(
                    &entry_path,
                    &dest_path,
                    app_handle,
                    phase,
                    total_files,
                    processed_files,
                    copied,
                )?;
            } else {
                fs::copy(&entry_path, &dest_path).map_err(|e| {
                    AppError::migration_error(
                        "copy_dir",
                        format!("Failed to copy file: {}", e),
                    )
                })?;

                *copied += 1;
                *processed_files += 1;

                // Emit progress every 10 files or for every file if total is small
                if (*copied).is_multiple_of(10) || total_files < 50 {
                    let status = MigrationStatus {
                        phase: phase.clone(),
                        current_file: Some(file_name.to_string_lossy().to_string()),
                        total_files,
                        processed_files: *processed_files,
                        error: None,
                    };
                    let _ = app_handle.emit("data-migration-progress", &status);
                }
            }
        }

        Ok(())
    }

    copy_dir_recursive(
        source,
        dest,
        app_handle,
        &phase,
        total_files,
        &mut processed_files,
        &mut copied,
    )?;

    Ok(copied)
}
