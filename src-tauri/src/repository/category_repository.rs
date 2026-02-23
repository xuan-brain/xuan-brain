//! Category repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{Category, CategoryNode, CreateCategory, UpdateCategory};
use crate::sys::error::{AppError, Result};
use tracing::info;

/// Repository for Category operations
pub struct CategoryRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> CategoryRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Get all categories as a flat list
    pub async fn find_all(&self) -> Result<Vec<Category>> {
        let result: Vec<Category> = self
            .db
            .query("SELECT * FROM category ORDER BY sort_order, name")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query categories: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} categories", result.len());
        Ok(result)
    }

    /// Get category tree structure
    pub async fn get_tree(&self) -> Result<Vec<CategoryNode>> {
        // Get root categories (no parent)
        let result: Vec<CategoryNode> = self
            .db
            .query(
                r#"
                SELECT *,
                       array::group(
                           SELECT *,
                                  array::group(
                                      SELECT * FROM category WHERE parent = $parent.id
                                  ) AS children
                           FROM category WHERE parent = $parent.id
                       ) AS children
                FROM category WHERE parent IS NONE
                ORDER BY sort_order, name
                "#,
            )
            .await
            .map_err(|e| AppError::generic(format!("Failed to query category tree: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }

    /// Find category by ID (string format like "category:123")
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Category>> {
        let id = id.to_string();
        let result: Vec<Category> = self
            .db
            .query("SELECT * FROM <record> $id LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get category: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new category
    pub async fn create(&self, category: CreateCategory) -> Result<Category> {
        let name = category.name.clone();
        let parent_id = category.parent_id.clone();

        // Use SurrealQL to create category with proper parent reference
        let query = if parent_id.is_some() {
            "CREATE category SET name = $name, parent = type::record($parent_id)"
        } else {
            "CREATE category SET name = $name, parent = NONE"
        };

        let result: Vec<Category> = self
            .db
            .query(query)
            .bind(("name", name))
            .bind(("parent_id", parent_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create category: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create category".to_string()))
    }

    /// Update category
    pub async fn update(&self, id: &str, update: UpdateCategory) -> Result<Category> {
        let id_owned = id.to_string();
        let mut sets = Vec::new();
        if update.name.is_some() {
            sets.push("name = $name");
        }
        if update.sort_order.is_some() {
            sets.push("sort_order = $sort_order");
        }

        if sets.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found("Category", id_owned));
        }

        let query = format!("UPDATE type::record($id) SET {}", sets.join(", "));

        let result: Vec<Category> = self
            .db
            .query(&query)
            .bind(("id", id_owned.clone()))
            .bind(("name", update.name))
            .bind(("sort_order", update.sort_order))
            .await
            .map_err(|e| AppError::generic(format!("Failed to update category: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("Category", id_owned))
    }

    /// Move category to a new parent
    pub async fn move_to(&self, id: &str, new_parent_id: Option<String>) -> Result<()> {
        let id = id.to_string();
        // Convert parent_id string to RecordId reference using type::record
        let query = if new_parent_id.is_some() {
            "UPDATE type::record($id) SET parent = type::record($parent)"
        } else {
            "UPDATE type::record($id) SET parent = NONE"
        };
        self.db
            .query(query)
            .bind(("id", id))
            .bind(("parent", new_parent_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to move category: {}", e)))?;

        Ok(())
    }

    /// Delete category and all its children
    pub async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        // Delete the category (cascade delete handled by SurrealDB graph relations)
        self.db
            .query("DELETE type::record($id)")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete category: {}", e)))?;

        Ok(())
    }

    /// Get all descendant categories
    pub async fn get_descendants(&self, id: &str) -> Result<Vec<Category>> {
        let id = id.to_string();
        let result: Vec<Category> = self
            .db
            .query(
                r#"
                SELECT * FROM category
                WHERE parent = type::record($id)
                   OR parent->parent = type::record($id)
                   OR parent->parent->parent = type::record($id)
                "#,
            )
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get descendants: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }

    /// Reorder categories
    pub async fn reorder(&self, orders: Vec<(String, i32)>) -> Result<()> {
        for (id, sort_order) in orders {
            self.db
                .query("UPDATE type::record($id) SET sort_order = $order")
                .bind(("id", id))
                .bind(("order", sort_order))
                .await
                .map_err(|e| AppError::generic(format!("Failed to reorder category: {}", e)))?;
        }

        Ok(())
    }

    /// Get max sort order for siblings (children of same parent)
    pub async fn get_max_sort_order(&self, parent_id: Option<&str>) -> Result<i32> {
        let result: Vec<i32> = match parent_id {
            Some(pid) => {
                let pid = pid.to_string();
                self.db
                    .query("SELECT VALUE sort_order FROM category WHERE parent = <record> $parent ORDER BY sort_order DESC LIMIT 1")
                    .bind(("parent", pid))
                    .await
                    .map_err(|e| AppError::generic(format!("Failed to get max sort order: {}", e)))?
                    .take(0)
                    .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?
            }
            None => {
                self.db
                    .query("SELECT VALUE sort_order FROM category WHERE parent = NONE ORDER BY sort_order DESC LIMIT 1")
                    .await
                    .map_err(|e| AppError::generic(format!("Failed to get max sort order: {}", e)))?
                    .take(0)
                    .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?
            }
        };

        Ok(result.into_iter().next().unwrap_or(-1) + 1)
    }

    /// Find all descendant category IDs recursively
    pub async fn find_all_descendant_ids(&self, id: &str) -> Result<Vec<String>> {
        let id_str = id.to_string();

        // Use a recursive approach to find all descendants
        let mut all_ids = vec![id_str.clone()];
        let mut to_process = vec![id_str];

        while let Some(current_id) = to_process.pop() {
            let children: Vec<String> = self
                .db
                .query("SELECT VALUE id FROM category WHERE parent = type::record($id)")
                .bind(("id", current_id))
                .await
                .map_err(|e| AppError::generic(format!("Failed to get children: {}", e)))?
                .take(0)
                .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

            for child_id in children {
                if !all_ids.contains(&child_id) {
                    all_ids.push(child_id.clone());
                    to_process.push(child_id);
                }
            }
        }

        Ok(all_ids)
    }

    /// Delete category and all descendants by ID
    pub async fn delete_with_descendants(&self, id: &str) -> Result<()> {
        let ids = self.find_all_descendant_ids(id).await?;

        for id in ids {
            self.db
                .query("DELETE <record> $id")
                .bind(("id", id))
                .await
                .map_err(|e| AppError::generic(format!("Failed to delete category: {}", e)))?;
        }

        Ok(())
    }

    /// Rebuild tree from structure (for drag-drop reordering)
    pub async fn rebuild_tree_from_structure(&self, nodes: &[TreeNodeData]) -> Result<()> {
        self.rebuild_tree_recursive(nodes, None, 0).await
    }

    async fn rebuild_tree_recursive(
        &self,
        nodes: &[TreeNodeData],
        parent_id: Option<String>,
        start_order: i32,
    ) -> Result<()> {
        for (index, node) in nodes.iter().enumerate() {
            let current_order = start_order + index as i32;

            // Update current node - use type::record to convert string to RecordId
            let query = if parent_id.is_some() {
                "UPDATE type::record($id) SET parent = type::record($parent), sort_order = $order"
            } else {
                "UPDATE type::record($id) SET parent = NONE, sort_order = $order"
            };
            self.db
                .query(query)
                .bind(("id", node.id.clone()))
                .bind(("parent", parent_id.clone()))
                .bind(("order", current_order))
                .await
                .map_err(|e| AppError::generic(format!("Failed to update category: {}", e)))?;

            // Recursively process children
            if let Some(children) = &node.children {
                if !children.is_empty() {
                    Box::pin(self.rebuild_tree_recursive(children, Some(node.id.clone()), 0)).await?;
                }
            }
        }
        Ok(())
    }
}

/// Tree node data for rebuilding tree structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeNodeData {
    pub id: String,
    pub name: String,
    pub children: Option<Vec<TreeNodeData>>,
}
