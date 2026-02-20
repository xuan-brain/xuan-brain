//! Attachment model for SurrealDB

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Attachment record representing a file attached to a paper
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Attachment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    /// Paper reference as string (e.g., "paper:123")
    pub paper: String,
    pub file_type: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new attachment
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateAttachment {
    pub paper_id: String,
    pub file_type: Option<String>,
    pub file_name: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
}

impl Attachment {
    pub fn new(paper: String, file_name: Option<String>) -> Self {
        Self {
            id: None,
            paper,
            file_type: None,
            file_name,
            file_path: None,
            file_size: None,
            created_at: Utc::now(),
        }
    }
}

impl From<CreateAttachment> for Attachment {
    fn from(create: CreateAttachment) -> Self {
        Self {
            id: None,
            paper: create.paper_id,
            file_type: create.file_type,
            file_name: create.file_name,
            file_path: create.file_path,
            file_size: create.file_size,
            created_at: Utc::now(),
        }
    }
}
