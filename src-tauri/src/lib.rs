mod command;
mod database;
mod sys;

use std::path::PathBuf;

use crate::command::label_command::{create_label, get_all_labels};
use crate::database::init_database_connection;
use crate::sys::error::Result;
use tauri::Manager;
use tracing::{error, info};

use crate::sys::dirs::init_app_dirs;
use crate::sys::log::init_logger;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    println!("Application starting...");
    // Initialize logger with console and file output
    // The WorkerGuard must be kept alive for the lifetime of the application

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwdwd| {}))
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize data directories on app startup
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                info!("Initializing application data directories...");
                let app_dirs = init_app_dirs().await;
                if let Err(err) = app_dirs {
                    error!("Failed to initialize application data directories: {}", err);
                    return;
                }
                let app_dirs = app_dirs.unwrap();
                let log_guard = init_logger(&PathBuf::from(&app_dirs.logs))
                    .await
                    .expect("Failed to initialize logger");
                info!("Logger initialized");
                app_handle.manage(log_guard);

                let db = init_database_connection(PathBuf::from(&app_dirs.data))
                    .await
                    .expect("Failed to initialize database connection");
                app_handle.manage(db);
                info!("Database connection initialized");
            });

            Ok(())
        })
        // TODO: Uncomment after fixing Tauri 2.x error type compatibility
        // .invoke_handler(tauri::generate_handler![get_all_labels])
        .invoke_handler(tauri::generate_handler![get_all_labels, create_label])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
