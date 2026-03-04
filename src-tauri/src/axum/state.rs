use std::sync::atomic::AtomicI64;
use std::sync::Arc;

use tauri::AppHandle;

use crate::database::DatabaseConnection;
use crate::sys::dirs::AppDirs;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub app_dirs: AppDirs,
    pub app_handle: Option<Arc<AppHandle>>,
    /// Currently selected category ID (-1 means none selected)
    pub selected_category_id: Arc<AtomicI64>,
}

impl AppState {
    pub fn new(db: Arc<DatabaseConnection>, app_dirs: AppDirs) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: None,
            selected_category_id: Arc::new(AtomicI64::new(-1)),
        }
    }

    pub fn new_with_handle(
        db: Arc<DatabaseConnection>,
        app_dirs: AppDirs,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: Some(Arc::new(app_handle)),
            selected_category_id: Arc::new(AtomicI64::new(-1)),
        }
    }
}
