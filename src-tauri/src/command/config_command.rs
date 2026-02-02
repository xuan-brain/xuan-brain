use crate::sys::config::AppConfig;
use crate::sys::dirs::AppDirs;
use crate::sys::error::Result;
use tauri::State;

#[tauri::command]
pub async fn get_app_config(app_dirs: State<'_, AppDirs>) -> Result<AppConfig> {
    AppConfig::load(&app_dirs.config)
}

#[tauri::command]
pub async fn save_app_config(app_dirs: State<'_, AppDirs>, config: AppConfig) -> Result<()> {
    config.save(&app_dirs.config)
}
