//! Category repository for SQLite using SeaORM

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, DatabaseConnection, sea_query::Expr};
use tracing::info;

use crate::database::entities::{category, paper_category};
use crate::models::{Category, CategoryNode, CreateCategory, UpdateCategory};
use crate::sys::error::{AppError, Result};

/// Repository for Category operations
pub struct CategoryRepository;

impl CategoryRepository {
    /// Find all categories
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Category>> {
        let categories = category::Entity::find()
            .order_by_asc(category::Column::SortOrder)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query categories: {}", e)))?;

        info!("Found {} categories", categories.len());
        Ok(categories.into_iter().map(Category::from).collect())
    }

    /// Find category by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Category>> {
        let cat = category::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get category: {}", e)))?;

        Ok(cat.map(Category::from))
    }

    /// Create a new category
    pub async fn create(db: &DatabaseConnection, create: CreateCategory) -> Result<Category> {
        let now = chrono::Utc::now();
        let new_category = category::ActiveModel {
            name: Set(create.name),
            parent_id: Set(create.parent_id),
            sort_order: Set(0),
            created_at: Set(now),
            ..Default::default()
        };

        let result = new_category
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create category: {}", e)))?;

        Ok(Category::from(result))
    }

    /// Update category
    pub async fn update(db: &DatabaseConnection, id: i64, update: UpdateCategory) -> Result<Category> {
        let cat = category::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find category: {}", e)))?
            .ok_or_else(|| AppError::not_found("Category", id.to_string()))?;

        let mut cat: category::ActiveModel = cat.into();
        if let Some(name) = update.name {
            cat.name = Set(name);
        }
        if let Some(sort_order) = update.sort_order {
            cat.sort_order = Set(sort_order);
        }

        let result = cat
            .update(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to update category: {}", e)))?;

        Ok(Category::from(result))
    }

    /// Delete category (cascade handled by foreign key)
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
        // First, move all child categories to root
        category::Entity::update_many()
            .filter(category::Column::ParentId.eq(id))
            .col_expr(category::Column::ParentId, Expr::value(Option::<i64>::None))
            .exec(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to update child categories: {}", e))
            })?;

        // Delete paper-category relations
        paper_category::Entity::delete_many()
            .filter(paper_category::Column::CategoryId.eq(id))
            .exec(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to delete category relations: {}", e))
            })?;

        // Delete the category
        category::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete category: {}", e)))?;

        Ok(())
    }

    /// Move category to a new parent
    pub async fn move_to_parent(
        db: &DatabaseConnection,
        id: i64,
        new_parent_id: Option<i64>,
    ) -> Result<()> {
        // Prevent moving to self
        if new_parent_id == Some(id) {
            return Err(AppError::validation(
                "parent_id",
                "Cannot move category to itself",
            ));
        }

        let cat = category::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find category: {}", e)))?
            .ok_or_else(|| AppError::not_found("Category", id.to_string()))?;

        let mut cat: category::ActiveModel = cat.into();
        cat.parent_id = Set(new_parent_id);
        cat.update(db).await.map_err(|e| {
            AppError::generic(format!("Failed to move category: {}", e))
        })?;

        Ok(())
    }

    /// Build tree structure from flat categories
    pub fn build_tree(categories: Vec<Category>) -> Vec<CategoryNode> {
        let nodes: Vec<CategoryNode> = categories.into_iter().map(CategoryNode::from).collect();
        build_tree_recursive(&nodes, None)
    }

    /// Load categories as tree structure
    pub async fn load_tree(db: &DatabaseConnection) -> Result<Vec<CategoryNode>> {
        let categories = Self::find_all(db).await?;
        Ok(Self::build_tree(categories))
    }

    /// Reorder categories
    pub async fn reorder(db: &DatabaseConnection, orders: Vec<(i64, i32)>) -> Result<()> {
        for (id, sort_order) in orders {
            let cat = category::Entity::find_by_id(id)
                .one(db)
                .await
                .map_err(|e| AppError::generic(format!("Failed to find category: {}", e)))?;

            if let Some(cat) = cat {
                let mut cat: category::ActiveModel = cat.into();
                cat.sort_order = Set(sort_order);
                cat.update(db).await.map_err(|e| {
                    AppError::generic(format!("Failed to reorder category: {}", e))
                })?;
            }
        }

        Ok(())
    }

    /// Rebuild tree from structure (for drag-drop reordering)
    pub async fn rebuild_tree_from_structure(
        db: &DatabaseConnection,
        nodes: &[TreeNodeData],
    ) -> Result<()> {
        Self::rebuild_tree_recursive(db, nodes, None, 0).await
    }

    async fn rebuild_tree_recursive(
        db: &DatabaseConnection,
        nodes: &[TreeNodeData],
        parent_id: Option<i64>,
        start_order: i32,
    ) -> Result<()> {
        for (index, node) in nodes.iter().enumerate() {
            let current_order = start_order + index as i32;

            // Update current node
            let cat = category::Entity::find_by_id(node.id)
                .one(db)
                .await
                .map_err(|e| AppError::generic(format!("Failed to find category: {}", e)))?;

            if let Some(cat) = cat {
                let mut cat: category::ActiveModel = cat.into();
                cat.parent_id = Set(parent_id);
                cat.sort_order = Set(current_order);
                cat.update(db).await.map_err(|e| {
                    AppError::generic(format!("Failed to update category: {}", e))
                })?;
            }

            // Recursively process children
            if !node.children.is_empty() {
                Box::pin(Self::rebuild_tree_recursive(db, &node.children, Some(node.id), 0)).await?;
            }
        }
        Ok(())
    }
}

/// Recursively build tree structure
fn build_tree_recursive(nodes: &[CategoryNode], parent_id: Option<i64>) -> Vec<CategoryNode> {
    let mut result = Vec::new();

    // Collect nodes with specified parent
    for node in nodes {
        if node.parent_id == parent_id {
            let mut node_clone = node.clone();
            // Recursively find children
            node_clone.children = build_tree_recursive(nodes, Some(node.id));
            result.push(node_clone);
        }
    }

    // Sort by sort_order
    result.sort_by_key(|n| n.sort_order);
    result
}

/// Tree node data for frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeNodeData {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub children: Vec<TreeNodeData>,
}
