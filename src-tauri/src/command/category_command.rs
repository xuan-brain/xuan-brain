use sea_orm::DatabaseConnection;
use tauri::State;

use crate::service::category_service::CategoryService;

// 我们假设 Tauri App 里通过 State 管理一个 DatabaseConnection
#[tauri::command]
pub async fn load_categories(
    db: State<'_, DatabaseConnection>,
) -> Result<Vec<CategoryDto>, String> {
    let service = CategoryService::new(db.inner());
    service.load_tree().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_category(
    db: State<'_, DatabaseConnection>,
    name: String,
    parent_path: Option<String>,
) -> Result<(), String> {
    let service = CategoryService::new(db.inner());
    service
        .create(&name, parent_path.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(
    db: State<'_, DatabaseConnection>,
    path: String,
) -> Result<(), String> {
    let service = CategoryService::new(db.inner());
    service
        .delete_by_path(&path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_category(
    db: State<'_, DatabaseConnection>,
    path: String,
    name: String,
) -> Result<(), String> {
    let service = CategoryService::new(db.inner());
    service
        .update_by_path(&path, &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn move_category(
    db: State<'_, DatabaseConnection>,
    dragged_path: String,
    target_path: Option<String>,
    position: String, // "above" | "below" | "child"
) -> Result<String, String> {
    let service = CategoryService::new(db.inner());
    service
        .move_node(&dragged_path, target_path.as_deref(), &position)
        .await
        .map_err(|e| e.to_string())
}

// 传给前端的 DTO，包含 path 字段（svelte-treeview 需要）
#[derive(serde::Serialize)]
pub struct CategoryDto {
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    pub sort_order: i64,
}
