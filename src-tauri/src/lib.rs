#![allow(dead_code)]
mod axum;
mod command;
mod llm;
mod papers;
mod repository;
mod service;
mod surreal;
mod sys;

use std::path::PathBuf;
use std::sync::Arc;

use crate::command::category_command::{
    create_category, delete_category, load_categories, move_category, reorder_tree, update_category,
};
use crate::command::clip_command::{create_clip, get_clip, list_clips};
use crate::command::config_command::{get_app_config, save_app_config};
use crate::command::data_folder_command::{
    get_data_folder_info_command, get_default_data_folder, migrate_data_folder_command,
    restart_app, revert_to_default_data_folder_command, validate_data_folder_command,
};
use crate::command::label_command::{create_label, delete_label, get_all_labels, update_label};
use crate::command::paper::{
    add_attachment, add_paper_label, delete_paper, get_all_papers, get_attachments,
    get_deleted_papers, get_paper, get_papers_by_category, get_pdf_attachment_path,
    import_paper_by_arxiv_id, import_paper_by_doi, import_paper_by_pdf, import_paper_by_pmid,
    migrate_abstract_field, open_paper_folder, permanently_delete_paper, read_pdf_as_blob,
    read_pdf_file, remove_paper_label, restore_paper, save_pdf_blob, save_pdf_with_annotations,
    update_paper_category, update_paper_details,
};
use crate::command::search_command::{search_papers, search_papers_with_score};
use crate::surreal::connection::{init_surreal_connection, SurrealClient};
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init());

    #[cfg(all(feature = "mcp-bridge", debug_assertions))]
    let builder = builder.plugin(tauri_plugin_mcp_bridge::init());

    builder
        .setup(move |app| {
            // Initialize data directories on app startup
            let app_handle = app.handle().clone();
            app_handle.manage(log_guard);
            app_handle.manage(app_dirs.clone());

            // Initialize SurrealDB only (no SQLite)
            let app_handle_for_axum = app.handle().clone();
            let app_dirs_for_surreal = app_dirs.clone();
            let data_dir = app_dirs_for_surreal.data.clone();

            let surreal_result = tauri::async_runtime::block_on(async move {
                init_surreal_connection(PathBuf::from(&data_dir)).await
            });

            match surreal_result {
                Ok(surreal_db) => {
                    info!("SurrealDB connection initialized");
                    let surreal_arc: Arc<SurrealClient> = Arc::new(surreal_db);
                    app_handle.manage(surreal_arc.clone());

                    // Start Axum API server with SurrealDB
                    crate::axum::start_axum_server_with_handle(
                        surreal_arc,
                        app_dirs_for_surreal,
                        app_handle_for_axum,
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to initialize SurrealDB connection: {}", e);
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
            get_app_config,
            save_app_config,
            // SurrealDB-based search commands
            search_papers,
            search_papers_with_score,
            // Data folder commands
            get_data_folder_info_command,
            get_default_data_folder,
            validate_data_folder_command,
            migrate_data_folder_command,
            revert_to_default_data_folder_command,
            restart_app,
            // Database migration commands
            migrate_abstract_field,
            // Clip commands
            list_clips,
            get_clip,
            create_clip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
