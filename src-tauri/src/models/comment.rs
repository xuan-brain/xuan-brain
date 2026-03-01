//! Comment domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::comment;

/// Comment embedded in a clipping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: i64,
    pub clipping_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComment {
    pub clipping_id: i64,
    pub content: String,
}

/// DTO for updating a comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateComment {
    pub content: String,
}

impl Comment {
    pub fn new(clipping_id: i64, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            clipping_id,
            content,
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<comment::Model> for Comment {
    fn from(model: comment::Model) -> Self {
        Self {
            id: model.id,
            clipping_id: model.clipping_id,
            content: model.content,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
