#![allow(dead_code)]
mod axum;
mod command;
mod database;
mod papers;
mod service;
mod sys;

use std::path::PathBuf;
use std::sync::Arc;

use crate::command::category_command::{
    create_category, delete_category, load_categories, move_category, reorder_tree, update_category,
};
use crate::command::config_command::{get_app_config, save_app_config};
use crate::command::label_command::{create_label, delete_label, get_all_labels, update_label};
use crate::command::paper_command::{
    add_attachment, add_paper_label, delete_paper, get_all_papers, get_attachments,
    get_deleted_papers, get_paper, get_papers_by_category, get_pdf_attachment_path,
    import_paper_by_arxiv_id, import_paper_by_doi, import_paper_by_pdf, import_paper_by_pmid,
    open_paper_folder, permanently_delete_paper, read_pdf_as_blob, read_pdf_file,
    remove_paper_label, restore_paper, save_pdf_blob, save_pdf_with_annotations,
    update_paper_category, update_paper_details,
};
use crate::database::init_database_connection;
use crate::sys::error::Result;
use futures::executor::block_on;
use tauri::Manager;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
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

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwdwd| {}))
        .plugin(tauri_plugin_tracing::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init());

    #[cfg(all(feature = "mcp-bridge", debug_assertions))]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    builder
        .setup(move |app| {
            // Initialize data directories on app startup
            let app_handle = app.handle().clone();
            app_handle.manage(log_guard);
            app_handle.manage(app_dirs.clone());

            // Initialize database connection synchronously in setup
            let app_handle = app.handle().clone();
            let app_dirs_for_db = app_dirs.clone();
            let app_dirs_for_axum = app_dirs.clone();
            let data_dir = app_dirs_for_db.data.clone();
            let db_result = tauri::async_runtime::block_on(async move {
                init_database_connection(PathBuf::from(&data_dir)).await
            });

            match db_result {
                Ok(db) => {
                    info!("Database connection initialized");
                    let db_arc = Arc::new(db);
                    app_handle.manage(db_arc.clone());
                    tauri::async_runtime::spawn(async move {
                        // Start Axum API server
                        crate::axum::start_axum_server(db_arc, app_dirs_for_axum);
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to initialize database connection: {}", e);
                    return Err(Box::new(e));
                }
            }

            // Setup system tray
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent window from closing and hide it instead
                window.hide().unwrap();
                api.prevent_close();
            }
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
            reorder_tree,
            get_all_papers,
            get_deleted_papers,
            get_papers_by_category,
            get_paper,
            import_paper_by_doi,
            import_paper_by_arxiv_id,
            import_paper_by_pdf,
            import_paper_by_pmid,
            add_paper_label,
            remove_paper_label,
            update_paper_details,
            update_paper_category,
            delete_paper,
            restore_paper,
            permanently_delete_paper,
            add_attachment,
            get_attachments,
            open_paper_folder,
            get_pdf_attachment_path,
            read_pdf_file,
            read_pdf_as_blob,
            save_pdf_blob,
            save_pdf_with_annotations,
            // save_pdf_file,
            // export_pdf_with_annotations,
            // save_annotations_data,
            // load_annotations_data,
            // save_pdf_with_annotations_data,
            get_app_config,
            save_app_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
