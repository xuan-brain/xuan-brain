use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tauri::State;
use tracing::{info, instrument};

use crate::service::category_service::CategoryService;
use crate::sys::error::Result;

#[tauri::command]
#[instrument(skip(db))]
pub async fn load_categories(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<CategoryDto>> {
    info!("Loading all categories");
    let service = CategoryService::new(db.inner().as_ref());
    let categories = service.load_tree().await?;
    info!("Loaded {} categories", categories.len());
    Ok(categories)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn create_category(
    db: State<'_, Arc<DatabaseConnection>>,
    name: String,
    parent_id: Option<i64>,
) -> Result<()> {
    info!(
        "Creating category '{}' with parent_id: {:?}",
        name, parent_id
    );
    let service = CategoryService::new(db.inner().as_ref());
    service.create(&name, parent_id).await?;
    info!("Category created successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_category(db: State<'_, Arc<DatabaseConnection>>, id: i64) -> Result<()> {
    info!("Deleting category with id={}", id);
    let service = CategoryService::new(db.inner().as_ref());
    service.delete_by_id(id).await?;
    info!("Category deleted successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn update_category(
    db: State<'_, Arc<DatabaseConnection>>,
    id: i64,
    name: String,
) -> Result<()> {
    info!("Updating category id={} to name '{}'", id, name);
    let service = CategoryService::new(db.inner().as_ref());
    service.update_by_id(id, &name).await?;
    info!("Category updated successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn move_category(
    db: State<'_, Arc<DatabaseConnection>>,
    dragged_id: i64,
    target_id: Option<i64>,
    position: String, // "above" | "below" | "child"
) -> Result<()> {
    info!(
        "Moving category {} to {:?} (position: {})",
        dragged_id, target_id, position
    );
    let service = CategoryService::new(db.inner().as_ref());
    service.move_node(dragged_id, target_id, &position).await?;
    info!("Category moved successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn reorder_tree(
    db: State<'_, Arc<DatabaseConnection>>,
    tree_data: Vec<TreeNodeDto>,
) -> Result<()> {
    info!(
        "Reordering tree based on new structure, {} root nodes",
        tree_data.len()
    );
    let service = CategoryService::new(db.inner().as_ref());
    service.rebuild_tree_from_structure(&tree_data).await?;
    info!("Tree reordered successfully");
    Ok(())
}

// 传给前端的 DTO
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CategoryDto {
    pub id: i64,
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
