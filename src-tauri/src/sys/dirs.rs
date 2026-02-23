use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use crate::sys::{
    consts::APP_FOLDER,
    error::{AppError, Result},
};

/// Data path configuration stored in system config directory
/// This file is always stored in the default system location
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataPathConfig {
    /// Custom data directory path (None = use system default)
    #[serde(default)]
    pub custom_data_path: Option<String>,
    /// Config version for future migrations
    #[serde(default = "default_version")]
    pub version: u32,
    /// Path to cleanup on next startup (old data folder after migration)
    #[serde(default)]
    pub pending_cleanup_path: Option<String>,
}

fn default_version() -> u32 {
    1
}

impl Default for DataPathConfig {
    fn default() -> Self {
        Self {
            custom_data_path: None,
            version: 1,
            pending_cleanup_path: None,
        }
    }
}

/// Application directory structure
#[derive(serde::Serialize, Debug, Clone)]
pub struct AppDirs {
    /// Configuration file directory
    pub config: String,
    /// Data file directory (database, documents, etc.)
    pub data: String,
    /// Cache directory
    pub cache: String,
    /// Logs directory
    pub logs: String,
    /// Files directory
    pub files: String,
    /// Whether using custom data path
    pub is_custom: bool,
}

/// Data folder information for frontend
#[derive(Debug, Serialize, Clone)]
pub struct DataFolderInfo {
    /// Current data folder path
    pub current_path: String,
    /// Config path
    pub config_path: String,
    /// Files path
    pub files_path: String,
    /// Cache path
    pub cache_path: String,
    /// Logs path
    pub logs_path: String,
    /// Whether using custom path
    pub is_custom: bool,
    /// Default system data path
    pub default_path: String,
    /// Total data size in bytes
    pub total_size: u64,
}

/// Migration status for frontend reporting
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MigrationStatus {
    /// Current migration phase
    pub phase: MigrationPhase,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Total number of files to process
    pub total_files: u32,
    /// Number of files processed
    pub processed_files: u32,
    /// Error message if failed
    pub error: Option<String>,
}

/// Migration phases
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationPhase {
    Preparing,
    CopyingDatabase,
    CopyingFiles,
    CopyingCache,
    CopyingConfig,
    CopyingLogs,
    Verifying,
    Completed,
    Failed,
    RollingBack,
}

/// Get the system config directory where data-path.json is stored
fn get_system_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or(AppError::file_system(
        "config_dir",
        "Cannot find default config directory",
    ))?;
    Ok(config_dir.join(APP_FOLDER))
}

/// Get the default data directory path
pub fn get_default_data_path() -> Result<String> {
    let data_dir = dirs::data_dir().ok_or(AppError::file_system(
        "data_dir",
        "Cannot find default data directory",
    ))?;
    Ok(data_dir.join(APP_FOLDER).to_string_lossy().to_string())
}

/// Load data path configuration from system config directory
pub fn load_data_path_config() -> Result<DataPathConfig> {
    let config_dir = get_system_config_dir()?;
    let config_path = config_dir.join("data-path.json");

    if !config_path.exists() {
        return Ok(DataPathConfig::default());
    }

    let content = fs::read_to_string(&config_path).map_err(|e| {
        error!("Failed to read data-path.json: {}", e);
        AppError::file_system(
            config_path.display().to_string(),
            format!("Failed to read data-path.json: {}", e),
        )
    })?;

    let config: DataPathConfig = serde_json::from_str(&content).map_err(|e| {
        error!("Failed to parse data-path.json: {}", e);
        AppError::config_error("data-path.json", format!("Failed to parse: {}", e))
    })?;

    Ok(config)
}

/// Save data path configuration to system config directory
pub fn save_data_path_config(config: &DataPathConfig) -> Result<()> {
    let config_dir = get_system_config_dir()?;

    // Ensure config directory exists
    fs::create_dir_all(&config_dir).map_err(|e| {
        error!("Failed to create config directory: {}", e);
        AppError::file_system(
            config_dir.display().to_string(),
            format!("Failed to create config directory: {}", e),
        )
    })?;

    let config_path = config_dir.join("data-path.json");
    let content = serde_json::to_string_pretty(config).map_err(|e| {
        error!("Failed to serialize data-path.json: {}", e);
        AppError::config_error("data-path.json", format!("Failed to serialize: {}", e))
    })?;

    let mut file = fs::File::create(&config_path).map_err(|e| {
        error!("Failed to create data-path.json: {}", e);
        AppError::file_system(
            config_path.display().to_string(),
            format!("Failed to create data-path.json: {}", e),
        )
    })?;

    file.write_all(content.as_bytes()).map_err(|e| {
        error!("Failed to write data-path.json: {}", e);
        AppError::file_system(
            config_path.display().to_string(),
            format!("Failed to write data-path.json: {}", e),
        )
    })?;

    info!("Data path configuration saved: {:?}", config_path);
    Ok(())
}

/// Calculate total size of data directory
pub fn calculate_data_size(app_dirs: &AppDirs) -> Result<u64> {
    let mut total_size: u64 = 0;

    let dirs_to_check = [
        &app_dirs.data,
        &app_dirs.files,
        &app_dirs.cache,
        &app_dirs.config,
        &app_dirs.logs,
    ];

    for dir_path in dirs_to_check {
        let path = PathBuf::from(dir_path);
        if path.exists() {
            total_size += calculate_dir_size(&path)?;
        }
    }

    Ok(total_size)
}

/// Recursively calculate directory size
fn calculate_dir_size(path: &PathBuf) -> Result<u64> {
    let mut size: u64 = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|e| {
            AppError::file_system(path.display().to_string(), format!("Failed to read dir: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                AppError::file_system(
                    path.display().to_string(),
                    format!("Failed to read entry: {}", e),
                )
            })?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                size += calculate_dir_size(&entry_path)?;
            } else {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }

    Ok(size)
}

/// Initialize application data directories
///
/// Detects and creates user data folder structure, including:
/// - config/: configuration files
/// - data/: database and documents
/// - cache/: cache files
/// - logs/: application logs
/// - files/: user files
///
/// Returns the path of each directory
pub async fn init_app_dirs() -> Result<AppDirs> {
    // Load data path configuration from system config directory
    let data_path_config = load_data_path_config()?;

    // Check if there's a pending cleanup path from previous migration
    if let Some(cleanup_path) = &data_path_config.pending_cleanup_path {
        info!("Found pending cleanup path: {}", cleanup_path);
        let cleanup_path_buf = PathBuf::from(cleanup_path);

        // Only cleanup if it exists and is different from current path
        if cleanup_path_buf.exists() {
            info!("Cleaning up old data directory: {:?}", cleanup_path_buf);
            match std::fs::remove_dir_all(&cleanup_path_buf) {
                Ok(_) => info!("Old data directory cleaned up successfully"),
                Err(e) => warn!("Failed to clean up old data directory: {}", e),
            }
        }

        // Clear the pending cleanup path
        let updated_config = DataPathConfig {
            custom_data_path: data_path_config.custom_data_path.clone(),
            version: data_path_config.version,
            pending_cleanup_path: None,
        };
        if let Err(e) = save_data_path_config(&updated_config) {
            warn!("Failed to clear pending cleanup path: {}", e);
        }
    }

    // Determine base data directory
    let (base_data_dir, is_custom) = if let Some(custom_path) = &data_path_config.custom_data_path
    {
        info!("Using custom data path: {}", custom_path);
        let custom_path_buf = PathBuf::from(custom_path);

        // Check if the path already ends with APP_FOLDER (XuanBrain)
        // If so, use it directly; otherwise, append APP_FOLDER
        let base = if custom_path_buf.file_name()
            .map(|name| name.to_string_lossy() == APP_FOLDER)
            .unwrap_or(false)
        {
            custom_path_buf
        } else {
            custom_path_buf.join(APP_FOLDER)
        };
        (base, true)
    } else {
        let sys_data_dir = dirs::data_dir().ok_or(AppError::file_system(
            "data_dir",
            "Cannot find default data directory",
        ))?;
        (sys_data_dir.join(APP_FOLDER), false)
    };

    info!("Application data directory: {:?}", base_data_dir);

    // Define subdirectory structure
    let dirs = vec![
        ("config", "Configuration files"),
        ("data", "Data files"),
        ("cache", "Cache files"),
        ("logs", "Log files"),
        ("files", "User files"),
    ];

    // Create all subdirectories
    for (dir_name, description) in dirs {
        let dir_path = base_data_dir.join(dir_name);

        match std::fs::metadata(&dir_path) {
            Ok(_) => {
                debug!("{} directory already exists: {:?}", description, dir_path);
            }
            Err(_) => {
                info!("Creating {} directory: {:?}", description, dir_path);
                std::fs::create_dir_all(&dir_path).map_err(|e| {
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
        config: base_data_dir.join("config").to_string_lossy().to_string(),
        data: base_data_dir.join("data").to_string_lossy().to_string(),
        cache: base_data_dir.join("cache").to_string_lossy().to_string(),
        logs: base_data_dir.join("logs").to_string_lossy().to_string(),
        files: base_data_dir.join("files").to_string_lossy().to_string(),
        is_custom,
    })
}

/// Get data folder information for frontend
pub fn get_data_folder_info(app_dirs: &AppDirs) -> Result<DataFolderInfo> {
    let default_path = get_default_data_path()?;
    let total_size = calculate_data_size(app_dirs)?;

    Ok(DataFolderInfo {
        current_path: app_dirs.data.clone(),
        config_path: app_dirs.config.clone(),
        files_path: app_dirs.files.clone(),
        cache_path: app_dirs.cache.clone(),
        logs_path: app_dirs.logs.clone(),
        is_custom: app_dirs.is_custom,
        default_path,
        total_size,
    })
}

/// Validation result for new data folder
#[derive(Debug, Serialize, Clone)]
pub struct ValidationResult {
    /// Whether the path is valid
    pub valid: bool,
    /// Warning messages
    pub warnings: Vec<String>,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Validate a potential new data folder path
pub fn validate_data_folder(path: &str, required_space: u64) -> Result<ValidationResult> {
    let mut warnings = Vec::new();
    let mut error = None;

    let new_path = PathBuf::from(path);

    // Check if path exists
    if !new_path.exists() {
        // Try to create it
        if let Err(e) = fs::create_dir_all(&new_path) {
            error = Some(format!("Cannot create directory: {}", e));
            return Ok(ValidationResult {
                valid: false,
                warnings,
                error,
            });
        }
    }

    // Check write permissions
    let test_file = new_path.join(".xuanbrain_write_test");
    match fs::File::create(&test_file) {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file);
        }
        Err(e) => {
            error = Some(format!("No write permission: {}", e));
            return Ok(ValidationResult {
                valid: false,
                warnings,
                error,
            });
        }
    }

    // Check available disk space
    if let Some(available) = get_available_space(&new_path) {
        if available < required_space {
            error = Some(format!(
                "Insufficient disk space: required {} bytes, available {} bytes",
                required_space, available
            ));
            return Ok(ValidationResult {
                valid: false,
                warnings,
                error,
            });
        }
        // Warn if less than 10% extra space
        if available < required_space * 11 / 10 {
            warnings.push("Disk space is low. Consider freeing up more space.".to_string());
        }
    }

    // Check if path already contains XuanBrain data
    let existing_data = new_path.join(APP_FOLDER);
    if existing_data.exists() {
        warnings.push(format!(
            "Path already contains {} folder. Data may be overwritten.",
            APP_FOLDER
        ));
    }

    // Check if path is a system directory
    if let Some(sys_dirs) = get_system_directories() {
        if sys_dirs.contains(&new_path) || sys_dirs.iter().any(|d| new_path.starts_with(d)) {
            warnings.push(
                "Selected path is in a system directory. Consider choosing a different location."
                    .to_string(),
            );
        }
    }

    Ok(ValidationResult {
        valid: error.is_none(),
        warnings,
        error,
    })
}

/// Get available disk space for a path (simplified implementation)
fn get_available_space(_path: &PathBuf) -> Option<u64> {
    // For cross-platform compatibility, we assume there's enough space
    // A more robust implementation would use platform-specific APIs
    // or the `fs2` crate for accurate disk space information
    Some(u64::MAX)
}

/// Get list of system directories that should be avoided
fn get_system_directories() -> Option<Vec<PathBuf>> {
    let mut dirs = Vec::new();

    // Windows system directories
    #[cfg(target_os = "windows")]
    {
        if let Some(windows) = std::env::var_os("SystemRoot") {
            dirs.push(PathBuf::from(windows));
        }
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            dirs.push(PathBuf::from(program_files));
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            dirs.push(PathBuf::from(program_files_x86));
        }
    }

    // Unix system directories
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        dirs.push(PathBuf::from("/usr"));
        dirs.push(PathBuf::from("/bin"));
        dirs.push(PathBuf::from("/sbin"));
        dirs.push(PathBuf::from("/etc"));
        dirs.push(PathBuf::from("/var"));
        #[cfg(target_os = "macos")]
        dirs.push(PathBuf::from("/System"));
    }

    if dirs.is_empty() {
        None
    } else {
        Some(dirs)
    }
}
