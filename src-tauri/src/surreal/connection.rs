//! SurrealDB connection management
//!
//! Provides initialization and connection handling for SurrealDB 3.0.
//!
//! ## Connection modes:
//! - **Debug mode**: Connects to remote SurrealDB server at 127.0.0.1:8000
//! - **Release mode**: Uses embedded RocksDB for persistent local storage

use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use surrealdb::engine::local::Db;
#[cfg(not(debug_assertions))]
use surrealdb::engine::local::RocksDb;

#[cfg(debug_assertions)]
use surrealdb::engine::remote::http::Client as HttpClient;
#[cfg(debug_assertions)]
use surrealdb::engine::remote::http::Http;

use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tracing::info;

use crate::sys::error::{AppError, Result};

/// Type alias for SurrealDB client
/// - Debug: Remote HTTP connection (Surreal<HttpClient>)
/// - Release: Local embedded RocksDB (Surreal<Db>)
#[cfg(debug_assertions)]
pub type SurrealClient = Surreal<HttpClient>;

#[cfg(not(debug_assertions))]
pub type SurrealClient = Surreal<Db>;

/// Debug mode connection settings
#[cfg(debug_assertions)]
const DEBUG_URL: &str = "127.0.0.1:8000";
#[cfg(debug_assertions)]
const DEBUG_USER: &str = "root";
#[cfg(debug_assertions)]
const DEBUG_PASS: &str = "secret";

/// Initialize SurrealDB connection
///
/// ## Debug mode (debug_assertions)
/// Connects to a remote SurrealDB server at `127.0.0.1:8000` with credentials:
/// - Username: `root`
/// - Password: `secret`
///
/// ## Release mode
/// Uses embedded RocksDB backend for persistent local storage.
/// Database files are stored in the provided data directory.
pub async fn init_surreal_connection(data_dir: PathBuf) -> Result<SurrealClient> {
    #[cfg(debug_assertions)]
    {
        let _ = data_dir; // Silence unused warning in debug mode
        init_surreal_remote().await
    }

    #[cfg(not(debug_assertions))]
    {
        init_surreal_embedded(data_dir).await
    }
}

/// Connect to remote SurrealDB server (debug mode only)
#[cfg(debug_assertions)]
async fn init_surreal_remote() -> Result<SurrealClient> {
    info!("Connecting to remote SurrealDB server at {}", DEBUG_URL);

    // Connect to remote SurrealDB server via HTTP
    // Surreal::new::<Http> returns Surreal<HttpClient>
    let db = Surreal::new::<Http>(DEBUG_URL.to_string())
        .await
        .map_err(|e| {
            AppError::generic(format!(
                "Failed to connect to remote SurrealDB at {}: {}",
                DEBUG_URL, e
            ))
        })?;

    // Sign in as root user
    db.signin(Root {
        username: DEBUG_USER.to_string(),
        password: DEBUG_PASS.to_string(),
    })
    .await
    .map_err(|e| AppError::generic(format!("Failed to authenticate with SurrealDB: {}", e)))?;

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

    info!(
        "Successfully connected to remote SurrealDB at {}",
        DEBUG_URL
    );
    Ok(db)
}

/// Initialize embedded SurrealDB with RocksDB (release mode only)
#[cfg(not(debug_assertions))]
async fn init_surreal_embedded(data_dir: PathBuf) -> Result<SurrealClient> {
    info!(
        "Initializing embedded SurrealDB with RocksDB at: {:?}",
        data_dir
    );

    // Initialize SurrealDB with RocksDB backend for persistent storage
    let db = Surreal::new::<RocksDb>(data_dir)
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

    info!("SurrealDB initialized successfully with RocksDB backend");
    Ok(db)
}
