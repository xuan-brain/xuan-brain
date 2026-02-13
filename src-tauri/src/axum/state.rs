use std::sync::Arc;

use sea_orm::DatabaseConnection;

use crate::sys::dirs::AppDirs;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseConnection>,
    pub app_dirs: AppDirs,
}

impl AppState {
    pub fn new(db: Arc<DatabaseConnection>, app_dirs: AppDirs) -> Self {
        Self { db, app_dirs }
    }
}
