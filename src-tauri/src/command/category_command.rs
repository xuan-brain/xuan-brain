use sea_orm::DatabaseConnection;
use tauri::State;
use tracing::{info, instrument};

use crate::service::category_service::CategoryService;
use crate::sys::error::Result;

// 我们假设 Tauri App 里通过 State 管理一个 DatabaseConnection
#[tauri::command]
#[instrument(skip(db))]
pub async fn load_categories(db: State<'_, DatabaseConnection>) -> Result<Vec<CategoryDto>> {
    info!("Loading all categories");
    let service = CategoryService::new(db.inner());
    let categories = service.load_tree().await?;
    info!("Loaded {} categories", categories.len());
    Ok(categories)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn create_category(
    db: State<'_, DatabaseConnection>,
    name: String,
    parent_path: Option<String>,
) -> Result<()> {
    info!(
        "Creating category '{}' with parent '{:?}'",
        name, parent_path
    );
    let service = CategoryService::new(db.inner());
    service.create(&name, parent_path.as_deref()).await?;
    info!("Category created successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_category(db: State<'_, DatabaseConnection>, path: String) -> Result<()> {
    info!("Deleting category at path '{}'", path);
    let service = CategoryService::new(db.inner());
    service.delete_by_path(&path).await?;
    info!("Category deleted successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn update_category(
    db: State<'_, DatabaseConnection>,
    path: String,
    name: String,
) -> Result<()> {
    info!("Updating category at path '{}' to name '{}'", path, name);
    let service = CategoryService::new(db.inner());
    service.update_by_path(&path, &name).await?;
    info!("Category updated successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn move_category(
    db: State<'_, DatabaseConnection>,
    dragged_path: String,
    target_path: Option<String>,
    position: String, // "above" | "below" | "child"
) -> Result<String> {
    info!(
        "Moving category '{}' to '{:?}' (position: {})",
        dragged_path, target_path, position
    );
    let service = CategoryService::new(db.inner());
    let new_path = service
        .move_node(&dragged_path, target_path.as_deref(), &position)
        .await?;
    info!("Category moved successfully to '{}'", new_path);
    Ok(new_path)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn reorder_tree(
    db: State<'_, DatabaseConnection>,
    tree_data: Vec<TreeNodeDto>,
) -> Result<()> {
    info!(
        "Reordering tree based on new structure, {} root nodes",
        tree_data.len()
    );
    let service = CategoryService::new(db.inner());
    service.rebuild_tree_from_structure(&tree_data).await?;
    info!("Tree reordered successfully");
    Ok(())
}

// 传给前端的 DTO，包含 path 字段（svelte-treeview 需要）
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CategoryDto {
    pub id: i64,
    pub path: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    pub sort_order: i64,
}

// 用于重建树的 DTO，包含完整的层级结构
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TreeNodeDto {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNodeDto>>,
}
