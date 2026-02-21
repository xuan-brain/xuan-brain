use std::sync::Arc;

use tauri::AppHandle;

use crate::surreal::connection::SurrealClient;
use crate::sys::dirs::AppDirs;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SurrealClient>,
    pub app_dirs: AppDirs,
    pub app_handle: Option<Arc<AppHandle>>,
}

impl AppState {
    pub fn new(db: Arc<SurrealClient>, app_dirs: AppDirs) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: None,
        }
    }

    pub fn new_with_handle(
        db: Arc<SurrealClient>,
        app_dirs: AppDirs,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: Some(Arc::new(app_handle)),
        }
    }
}
