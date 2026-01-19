use tracing::{debug, error, info};

use crate::sys::{
    consts::APP_FOLDER,
    error::{AppError, Result},
};

/// Application directory structure
#[derive(serde::Serialize, Debug)]
pub struct AppDirs {
    /// Configuration file directory
    pub config: String,
    /// Data file directory (database, documents, etc.)
    pub data: String,
    /// Cache directory
    pub cache: String,
    /// Logs directory
    pub logs: String,
}

/// Initialize application data directories
///
/// Detects and creates user data folder structure, including:
/// - config/: configuration files
/// - data/: database and documents
/// - cache/: cache files
/// - logs/: application logs
///
/// Returns the path of each directory
pub async fn init_app_dirs() -> Result<AppDirs> {
    // Get application data directory
    let sys_data_dir = dirs::data_dir().ok_or(AppError::file_system(
        "data_dir",
        "cannot find default data dir",
    ))?;
    let data_dir = sys_data_dir.join(APP_FOLDER);

    info!("Application data directory: {:?}", data_dir);

    // Define subdirectory structure
    let dirs = vec![
        ("config", "Configuration files"),
        ("data", "Data files"),
        ("cache", "Cache files"),
        ("logs", "Log files"),
    ];

    // Create all subdirectories
    for (dir_name, description) in dirs {
        let dir_path = data_dir.join(dir_name);

        match tokio::fs::metadata(&dir_path).await {
            Ok(_) => {
                debug!("{} directory already exists: {:?}", description, dir_path);
            }
            Err(_) => {
                info!("Creating {} directory: {:?}", description, dir_path);
                tokio::fs::create_dir_all(&dir_path).await.map_err(|e| {
                    error!("Failed to create directory: {} ({})", dir_path.display(), e);
                    AppError::file_system(
                        dir_path.display().to_string(),
                        format!("Failed to create {} directory", description),
                    )
                })?;
                info!(
                    "{} directory created successfully: {:?}",
                    description, dir_path
                );
            }
        }
    }

    // Return all directory paths
    Ok(AppDirs {
        config: data_dir.join("config").to_string_lossy().to_string(),
        data: data_dir.join("data").to_string_lossy().to_string(),
        cache: data_dir.join("cache").to_string_lossy().to_string(),
        logs: data_dir.join("logs").to_string_lossy().to_string(),
    })
}
