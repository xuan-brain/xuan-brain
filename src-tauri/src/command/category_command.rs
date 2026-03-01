use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::{CreateCategory, UpdateCategory};
use crate::repository::{CategoryRepository, TreeNodeData};
use crate::sys::error::Result;

#[tauri::command]
#[instrument(skip(db))]
pub async fn load_categories(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<CategoryDto>> {
    info!("Loading all categories");
    let categories = CategoryRepository::find_all(&db).await?;

    let result: Vec<CategoryDto> = categories
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id.to_string(),
            name: c.name,
            parent_id: c.parent_id.map(|id| id.to_string()),
            sort_order: c.sort_order,
        })
        .collect();

    info!("Loaded {} categories", result.len());
    Ok(result)
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn create_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    name: String,
    parent_id: Option<String>,
) -> Result<()> {
    info!(
        "Creating category '{}' with parent_id: {:?}",
        name, parent_id
    );

    let parent_id_num = parent_id
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|_| crate::sys::error::AppError::validation("parent_id", "Invalid parent_id format"))?;

    let create_data = CreateCategory {
        name: name.clone(),
        parent_id: parent_id_num,
    };

    CategoryRepository::create(&db, create_data).await?;

    let _ = app
        .notification()
        .builder()
        .title("Category Created")
        .body(format!("Category '{}' created successfully", name))
        .show();

    info!("Category created successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn delete_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
) -> Result<()> {
    info!("Deleting category with id={}", id);

    let id_num = id
        .parse::<i64>()
        .map_err(|_| crate::sys::error::AppError::validation("id", "Invalid id format"))?;

    CategoryRepository::delete(&db, id_num).await?;

    let _ = app
        .notification()
        .builder()
        .title("Category Deleted")
        .body(format!("Category with id {} deleted successfully", id))
        .show();

    info!("Category deleted successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: String,
    name: String,
) -> Result<()> {
    info!("Updating category id={} to name '{}'", id, name);

    let id_num = id
        .parse::<i64>()
        .map_err(|_| crate::sys::error::AppError::validation("id", "Invalid id format"))?;

    CategoryRepository::update(
        &db,
        id_num,
        UpdateCategory {
            name: Some(name.clone()),
            sort_order: None,
        },
    )
    .await?;

    let _ = app
        .notification()
        .builder()
        .title("Category Updated")
        .body(format!("Category updated to '{}'", name))
        .show();

    info!("Category updated successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn move_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    dragged_id: String,
    target_id: Option<String>,
    position: String, // "above" | "below" | "child"
) -> Result<()> {
    info!(
        "Moving category {} to {:?} (position: {})",
        dragged_id, target_id, position
    );

    let dragged_id_num = dragged_id
        .parse::<i64>()
        .map_err(|_| crate::sys::error::AppError::validation("dragged_id", "Invalid id format"))?;

    // Determine new parent based on position
    let new_parent_id = match (target_id.as_ref(), position.as_str()) {
        (Some(tid), "child") => {
            Some(tid.parse::<i64>().map_err(|_| {
                crate::sys::error::AppError::validation("target_id", "Invalid id format")
            })?)
        }
        (Some(tid), "above" | "below") => {
            // Get target's parent
            let target_id_num = tid.parse::<i64>().map_err(|_| {
                crate::sys::error::AppError::validation("target_id", "Invalid id format")
            })?;
            if let Some(target) = CategoryRepository::find_by_id(&db, target_id_num).await? {
                target.parent_id
            } else {
                None
            }
        }
        _ => None,
    };

    CategoryRepository::move_to_parent(&db, dragged_id_num, new_parent_id).await?;

    let _ = app
        .notification()
        .builder()
        .title("Category Moved")
        .body("Category structure updated successfully")
        .show();

    info!("Category moved successfully");
    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn reorder_tree(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    tree_data: Vec<TreeNodeDto>,
) -> Result<()> {
    info!(
        "Reordering tree based on new structure, {} root nodes",
        tree_data.len()
    );

    // Convert TreeNodeDto to TreeNodeData
    let nodes = convert_tree_nodes(&tree_data);

    CategoryRepository::rebuild_tree_from_structure(&db, &nodes).await?;

    let _ = app
        .notification()
        .builder()
        .title("Categories Reordered")
        .body("Category tree reordered successfully")
        .show();

    info!("Tree reordered successfully");
    Ok(())
}

/// Convert TreeNodeDto to TreeNodeData recursively
fn convert_tree_nodes(dtos: &[TreeNodeDto]) -> Vec<TreeNodeData> {
    dtos.iter()
        .map(|n| TreeNodeData {
            id: n.id.parse::<i64>().unwrap_or(0),
            name: n.name.clone(),
            children: convert_tree_nodes(&n.children),
        })
        .collect()
}

// DTO for frontend
#[derive(Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub sort_order: i32,
}

// DTO for tree rebuilding, includes full hierarchy
#[derive(Serialize, Deserialize, Debug)]
pub struct TreeNodeDto {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub children: Vec<TreeNodeDto>,
}
