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
            .query("SELECT * FROM type::thing($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get category: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new category
    pub async fn create(&self, category: CreateCategory) -> Result<Category> {
        let category = Category::from(category);

        let result: Vec<Category> = self
            .db
            .query("CREATE category CONTENT $category")
            .bind(("category", category))
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

        let query = format!("UPDATE type::thing($id) SET {}", sets.join(", "));

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
        self.db
            .query("UPDATE type::thing($id) SET parent = $parent")
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
            .query("DELETE type::thing($id)")
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
                WHERE parent = type::thing($id)
                   OR parent->parent = type::thing($id)
                   OR parent->parent->parent = type::thing($id)
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
                .query("UPDATE type::thing($id) SET sort_order = $order")
                .bind(("id", id))
                .bind(("order", sort_order))
                .await
                .map_err(|e| AppError::generic(format!("Failed to reorder category: {}", e)))?;
        }

        Ok(())
    }
}
