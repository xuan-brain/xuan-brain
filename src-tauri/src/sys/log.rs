use crate::sys::error::{AppError, Result};
use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

/// Initialize the application logger with console and file output
///
/// # Arguments
///
/// * `log_dir` - The directory where log files will be stored
///
/// # Returns
///
/// Returns a `WorkerGuard` that must be kept alive for the lifetime of the application
/// to ensure logs are flushed to file properly.
///
/// # Log Rotation
///
/// Log files are rotated weekly. Each file is named with the format: `xuan-brain.YYYY-Www.log`
/// where YYYY is the year and ww is the ISO week number.
///
/// # Log Format
///
/// Console output: Colored, human-readable format
/// File output: Detailed format with timestamps, file location, and span information
pub async fn init_logger(log_dir: &PathBuf) -> Result<WorkerGuard> {
    // Ensure log directory exists
    tokio::fs::create_dir_all(log_dir).await.map_err(|_e| {
        AppError::file_system(
            log_dir.display().to_string(),
            "Failed to create log directory",
        )
    })?;

    // Create file appender with weekly rotation
    // This will create files like: xuan-brain.2024-W03.log, xuan-brain.2024-W04.log, etc.
    let file_appender = tracing_appender::rolling::weekly(log_dir, "xuan-brain");
    let (non_blocking_file_appender, file_guard) = tracing_appender::non_blocking(file_appender);

    // Set up environment filter from RUST_LOG environment variable
    // Default to debug level if not set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("xuan_brain=debug,tauri=debug"));

    // Console layer with colored output and span events
    let console_layer = fmt::layer()
        // .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(true)
        .with_filter(env_filter.clone());

    // File layer with more detailed formatting
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file_appender)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false) // No colors in file
        .with_filter(env_filter);

    // Initialize global subscriber with both console and file layers
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(file_guard)
}

/// Initialize the application logger with custom log level
///
/// # Arguments
///
/// * `log_dir` - The directory where log files will be stored
/// * `log_level` - The default log level (e.g., "debug", "info", "warn", "error")
///
/// # Returns
///
/// Returns a `WorkerGuard` that must be kept alive for the lifetime of the application
pub async fn init_logger_with_level(log_dir: &PathBuf, log_level: &str) -> Result<WorkerGuard> {
    // Ensure log directory exists
    tokio::fs::create_dir_all(log_dir).await.map_err(|_e| {
        AppError::file_system(
            log_dir.display().to_string(),
            "Failed to create log directory",
        )
    })?;

    // Create file appender with weekly rotation
    let file_appender = tracing_appender::rolling::weekly(log_dir, "xuan-brain");
    let (non_blocking_file_appender, file_guard) = tracing_appender::non_blocking(file_appender);

    // Set up environment filter from RUST_LOG environment variable
    // Fall back to the provided log_level if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!("xuan_brain={},tauri={}", log_level, log_level))
    });

    // Console layer with colored output
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(true)
        .with_filter(env_filter.clone());

    // File layer
    let file_layer = fmt::layer()
        .with_writer(non_blocking_file_appender)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false)
        .with_filter(env_filter);

    // Initialize global subscriber
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    Ok(file_guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_logger() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let log_dir = temp_dir.path().to_path_buf();

        // Initialize logger
        let _guard = init_logger(&log_dir)
            .await
            .expect("Failed to initialize logger");

        // Log some test messages
        tracing::info!("Test info message");
        tracing::debug!("Test debug message");
        tracing::warn!("Test warning message");
        tracing::error!("Test error message");

        // The guard will be dropped here, flushing logs
    }

    #[tokio::test]
    async fn test_init_logger_with_level() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let log_dir = temp_dir.path().to_path_buf();

        // Initialize logger with info level
        let _guard = init_logger_with_level(&log_dir, "info")
            .await
            .expect("Failed to initialize logger");

        // Log some test messages
        tracing::info!("Test info message");
        tracing::debug!("This debug message should not appear");
    }
}
