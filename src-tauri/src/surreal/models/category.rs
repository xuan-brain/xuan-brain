//! Category model for SurrealDB

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Category record representing a hierarchical category
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Category {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    /// Parent category reference as RecordId
    pub parent: Option<RecordId>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new category
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateCategory {
    pub name: String,
    pub parent_id: Option<String>,
}

/// DTO for updating a category
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

/// Category node with children for tree structure
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CategoryNode {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub parent: Option<RecordId>,
    pub sort_order: i32,
    pub children: Vec<CategoryNode>,
}

impl Category {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            parent: None,
            sort_order: 0,
            created_at: Utc::now(),
        }
    }
}
