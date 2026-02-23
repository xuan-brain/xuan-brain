use std::sync::Arc;

use serde::{Deserialize, Serialize};
use surrealdb_types::RecordIdKey;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::repository::{CategoryRepository, TreeNodeData};
use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateCategory, UpdateCategory};
use crate::sys::error::Result;

/// Convert RecordId to string
fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    format!("{}:{}", id.table, record_id_key_to_string(&id.key))
}

fn record_id_key_to_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        RecordIdKey::Array(_) => "array".to_string(),
        RecordIdKey::Object(_) => "object".to_string(),
        RecordIdKey::Range(_) => "range".to_string(),
    }
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn load_categories(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<CategoryDto>> {
    info!("Loading all categories");
    let repo = CategoryRepository::new(&db);
    let categories = repo.find_all().await?;

    let result: Vec<CategoryDto> = categories
        .into_iter()
        .map(|c| CategoryDto {
            id: c.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
            name: c.name,
            parent_id: c.parent.map(|rid| record_id_to_string(&rid)),
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
    db: State<'_, Arc<SurrealClient>>,
    name: String,
    parent_id: Option<String>,
) -> Result<()> {
    info!(
        "Creating category '{}' with parent_id: {:?}",
        name, parent_id
    );
    let repo = CategoryRepository::new(&db);

    // Get max sort order for siblings
    let _sort_order = repo.get_max_sort_order(parent_id.as_deref()).await?;

    let create_data = CreateCategory {
        name: name.clone(),
        parent_id: parent_id.clone(),
    };

    repo.create(create_data).await?;

    let _ = app.notification()
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
    db: State<'_, Arc<SurrealClient>>,
    id: String,
) -> Result<()> {
    info!("Deleting category with id={}", id);
    let repo = CategoryRepository::new(&db);
    repo.delete_with_descendants(&id).await?;

    let _ = app.notification()
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
    db: State<'_, Arc<SurrealClient>>,
    id: String,
    name: String,
) -> Result<()> {
    info!("Updating category id={} to name '{}'", id, name);
    let repo = CategoryRepository::new(&db);
    repo.update(&id, UpdateCategory {
        name: Some(name.clone()),
        sort_order: None,
    }).await?;

    let _ = app.notification()
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
    db: State<'_, Arc<SurrealClient>>,
    dragged_id: String,
    target_id: Option<String>,
    position: String, // "above" | "below" | "child"
) -> Result<()> {
    info!(
        "Moving category {} to {:?} (position: {})",
        dragged_id, target_id, position
    );
    let repo = CategoryRepository::new(&db);

    // Determine new parent based on position
    let new_parent_id = match (target_id.as_ref(), position.as_str()) {
        (Some(tid), "child") => Some(tid.clone()),
        (Some(tid), "above" | "below") => {
            // Get target's parent
            if let Some(target) = repo.find_by_id(tid).await? {
                target.parent.map(|rid| record_id_to_string(&rid))
            } else {
                None
            }
        }
        _ => None,
    };

    repo.move_to(&dragged_id, new_parent_id).await?;

    let _ = app.notification()
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
    db: State<'_, Arc<SurrealClient>>,
    tree_data: Vec<TreeNodeDto>,
) -> Result<()> {
    info!(
        "Reordering tree based on new structure, {} root nodes",
        tree_data.len()
    );
    let repo = CategoryRepository::new(&db);

    // Convert TreeNodeDto to TreeNodeData
    let nodes: Vec<TreeNodeData> = tree_data.iter().map(|n| TreeNodeData {
        id: n.id.clone(),
        name: n.name.clone(),
        children: n.children.as_ref().map(|c| c.iter().map(|child| TreeNodeData {
            id: child.id.clone(),
            name: child.name.clone(),
            children: child.children.as_ref().map(|cc| cc.iter().map(|ccc| TreeNodeData {
                id: ccc.id.clone(),
                name: ccc.name.clone(),
                children: None,
            }).collect()),
        }).collect()),
    }).collect();

    repo.rebuild_tree_from_structure(&nodes).await?;

    let _ = app.notification()
        .builder()
        .title("Categories Reordered")
        .body("Category tree reordered successfully")
        .show();

    info!("Tree reordered successfully");
    Ok(())
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNodeDto>>,
}
