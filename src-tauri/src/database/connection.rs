//! SQLite connection management using SeaORM
//!
//! Provides initialization and connection handling for SQLite database.

use std::path::PathBuf;
use std::sync::Arc;

use sea_orm::{Database, DatabaseConnection};
use tracing::info;

use crate::database::migration::run_migrations;
use crate::sys::error::{AppError, Result};

/// Initialize SQLite connection
///
/// Creates or connects to the SQLite database file at `{data_dir}/xuan-brain.sqlite`.
/// Runs any pending migrations automatically.
pub async fn init_sqlite_connection(data_dir: PathBuf) -> Result<Arc<DatabaseConnection>> {
    let db_path = data_dir.join("xuan-brain.sqlite");
    let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

    info!("Connecting to SQLite database at: {:?}", db_path);

    let db = Database::connect(&db_url)
        .await
        .map_err(|e| AppError::generic(format!("Failed to connect to SQLite: {}", e)))?;

    info!("SQLite connection established");

    // Run migrations
    run_migrations(&db)
        .await
        .map_err(|e| AppError::generic(format!("Failed to run migrations: {}", e)))?;

    info!("Database migrations completed");

    Ok(Arc::new(db))
}
