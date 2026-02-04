#![allow(unused_imports)]
pub mod entities;

use std::{path::PathBuf, time::Duration};

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::info;

use crate::sys::error::{AppError, Result};

pub async fn init_database_connection(data_dir: PathBuf) -> Result<DatabaseConnection> {
    let db_path = data_dir.join("xuan_brain.sqlite");
    // For SQLite on Unix-like systems, ensure the directory exists first
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                parent.display().to_string(),
                format!("Failed to create database directory: {}", e),
            )
        })?;
    }
    
    // Use file:// URL format for proper handling of spaces
    let db_url = format!("sqlite:file://{}?mode=rwc", db_path.display());
    info!("database url: {db_url}");
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true); // enable SQLx logging for debugging

    let db = Database::connect(opt).await.map_err(|e| {
        tracing::error!("Database connection failed: {}", e);
        AppError::from(e)
    })?;
    info!("Database connection initialized");
    Migrator::up(&db, None).await.map_err(AppError::from)?;
    info!("Database migration completed");
    Ok(db)
}
