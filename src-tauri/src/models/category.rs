//! Category domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::category;

/// Category record representing a hierarchical category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub parent_id: Option<i64>,
}

/// DTO for updating a category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

/// Category node with children for tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryNode {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    #[serde(default)]
    pub children: Vec<CategoryNode>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            id: 0,
            name,
            parent_id: None,
            sort_order: 0,
            created_at: Utc::now(),
        }
    }
}

impl From<category::Model> for Category {
    fn from(model: category::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            sort_order: model.sort_order,
            created_at: model.created_at,
        }
    }
}

impl From<Category> for CategoryNode {
    fn from(category: Category) -> Self {
        Self {
            id: category.id,
            name: category.name,
            parent_id: category.parent_id,
            sort_order: category.sort_order,
            children: Vec::new(),
        }
    }
}
