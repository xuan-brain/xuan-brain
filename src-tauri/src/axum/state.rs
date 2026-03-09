use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tauri::AppHandle;

use crate::database::DatabaseConnection;
use crate::sys::dirs::AppDirs;

/// Shared state for selected category ID
/// Used by both Tauri commands and Axum handlers
#[derive(Clone, Default)]
pub struct SelectedCategoryState {
    /// Currently selected category ID (-1 means none selected)
    pub category_id: Arc<AtomicI64>,
}

impl SelectedCategoryState {
    pub fn new() -> Self {
        Self {
            category_id: Arc::new(AtomicI64::new(-1)),
        }
    }

    /// Set the selected category ID
    pub fn set(&self, id: Option<i64>) {
        let value = id.unwrap_or(-1);
        self.category_id.store(value, Ordering::SeqCst);
    }

    /// Get the selected category ID (None means no selection)
    pub fn get(&self) -> Option<i64> {
        let value = self.category_id.load(Ordering::SeqCst);
        if value < 0 {
            None
        } else {
            Some(value)
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub app_dirs: AppDirs,
    pub app_handle: Option<Arc<AppHandle>>,
    /// Shared selected category state
    pub selected_category: SelectedCategoryState,
}

impl AppState {
    pub fn new(db: Arc<DatabaseConnection>, app_dirs: AppDirs) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: None,
            selected_category: SelectedCategoryState::new(),
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
            selected_category: SelectedCategoryState::new(),
        }
    }

    /// Create AppState with shared selected category state
    pub fn new_with_selected_category(
        db: Arc<DatabaseConnection>,
        app_dirs: AppDirs,
        app_handle: AppHandle,
        selected_category: SelectedCategoryState,
    ) -> Self {
        Self {
            db,
            app_dirs,
            app_handle: Some(Arc::new(app_handle)),
            selected_category,
        }
    }
}
