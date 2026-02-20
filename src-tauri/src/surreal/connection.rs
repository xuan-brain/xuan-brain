//! SurrealDB connection management
//!
//! Provides initialization and connection handling for SurrealDB 3.0
//! in embedded mode with RocksDB persistence.

use std::path::PathBuf;

use surrealdb::engine::local::Db;
use surrealdb::engine::local::RocksDb;
use surrealdb::Surreal;
use tracing::info;

use crate::sys::error::{AppError, Result};

/// Type alias for SurrealDB client
pub type SurrealClient = Surreal<Db>;

/// Initialize SurrealDB connection with RocksDB persistent storage
///
/// Uses kv-rocksdb backend for persistent storage. The database files
/// are stored in the `surrealdb` subdirectory alongside the SQLite database.
pub async fn init_surreal_connection(data_dir: PathBuf) -> Result<SurrealClient> {
    // Create surrealdb directory for RocksDB storage
    // let db_path = data_dir.join("surrealdb");
    // std::fs::create_dir_all(&db_path).map_err(|e| {
    //     AppError::file_system(
    //         db_path.display().to_string(),
    //         format!("Failed to create SurrealDB directory: {}", e),
    //     )
    // })?;

    info!("Initializing SurrealDB with RocksDB at: {:?}", data_dir);

    // Initialize SurrealDB with RocksDB backend for persistent storage
    let db: Surreal<Db> = Surreal::new::<RocksDb>(data_dir)
        .await
        .map_err(|e| AppError::generic(format!("Failed to create SurrealDB instance: {}", e)))?;

    // Select namespace and database
    db.use_ns("xuanbrain")
        .use_db("papers")
        .await
        .map_err(|e| AppError::generic(format!("Failed to select namespace/database: {}", e)))?;

    // Execute schema definition (this is idempotent - safe to run on existing database)
    let schema = include_str!("../schema.surql");
    db.query(schema)
        .await
        .map_err(|e| AppError::generic(format!("Failed to execute schema: {}", e)))?;

    info!("SurrealDB 3.0 initialized successfully with RocksDB backend");
    Ok(db)
}

#[cfg(test)]
pub async fn init_surreal_memory() -> Result<SurrealClient> {
    use surrealdb::engine::local::Mem;

    let db: Surreal<Db> = Surreal::new::<Mem>(())
        .await
        .map_err(|e| AppError::generic(format!("Failed to create in-memory database: {}", e)))?;

    db.use_ns("test")
        .use_db("test")
        .await
        .map_err(|e| AppError::generic(format!("Failed to select namespace/database: {}", e)))?;

    // Execute schema definition for tests
    let schema = include_str!("../schema.surql");
    db.query(schema)
        .await
        .map_err(|e| AppError::generic(format!("Failed to execute schema in test: {}", e)))?;

    Ok(db)
}
