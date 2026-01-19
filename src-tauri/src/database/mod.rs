mod entities;
mod migrations;

use std::{path::PathBuf, time::Duration};

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::info;

use crate::sys::error::{AppError, Result};

pub async fn init_database_connection(data_dir: PathBuf) -> Result<DatabaseConnection> {
    let db_url = format!(
        "sqlite://{}?mode=rwc",
        data_dir.join("xuan_brain.sqlite").to_str().unwrap()
    );
    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(false) // disable SQLx logging
        .set_schema_search_path("my_schema"); // set default Postgres schema

    let db = Database::connect(opt)
        .await
        .map_err(|e| AppError::database(e.to_string()))?;
    info!("Database connection initialized");
    Ok(db)
}
