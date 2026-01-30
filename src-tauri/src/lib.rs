#![allow(dead_code)]
mod command;
mod database;
mod papers;
mod service;
mod sys;

use std::path::PathBuf;

use crate::command::category_command::{
    create_category, delete_category, load_categories, move_category, update_category,
};
use crate::command::label_command::{create_label, delete_label, get_all_labels, update_label};
use crate::command::paper_command::{
    add_paper_label, delete_paper, get_all_papers, get_deleted_papers, get_paper,
    get_papers_by_category, import_paper_by_arxiv_id, import_paper_by_doi,
    permanently_delete_paper, remove_paper_label, restore_paper, update_paper_category,
    update_paper_details,
};
use crate::database::init_database_connection;
use crate::sys::error::Result;
use futures::executor::block_on;
use tauri::Manager;
use tracing::info;

use crate::sys::dirs::init_app_dirs;
use crate::sys::log::init_logger;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    println!("Application starting...");
    println!("Initializing application data directories...");

    let app_dirs =
        block_on(init_app_dirs()).expect("Failed to initialize application data directories");
    println!("Application data directories initialized");
    println!("Initializing logger...");
    let (log_guard, layer) =
        block_on(init_logger(&PathBuf::from(&app_dirs.logs))).expect("Failed to initialize logger");
    info!("Logger initialized");
    tracing::subscriber::set_global_default(layer)
        .expect("failed to set global default subscriber");

    // Initialize logger with console and file output
    // The WorkerGuard must be kept alive for the lifetime of the application

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwdwd| {}))
        .plugin(tauri_plugin_tracing::Builder::new().build())
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
            app_handle.manage(log_guard);
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let db = init_database_connection(PathBuf::from(&app_dirs.data))
                    .await
                    .expect("Failed to initialize database connection");
                info!("Database connection initialized");
                app_handle.manage(db);
            });
            Ok(())
        })
        // TODO: Uncomment after fixing Tauri 2.x error type compatibility
        // .invoke_handler(tauri::generate_handler![get_all_labels])
        .invoke_handler(tauri::generate_handler![
            get_all_labels,
            create_label,
            delete_label,
            update_label,
            load_categories,
            create_category,
            delete_category,
            update_category,
            move_category,
            get_all_papers,
            get_deleted_papers,
            get_papers_by_category,
            get_paper,
            import_paper_by_doi,
            import_paper_by_arxiv_id,
            add_paper_label,
            remove_paper_label,
            update_paper_details,
            update_paper_category,
            delete_paper,
            restore_paper,
            permanently_delete_paper
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
