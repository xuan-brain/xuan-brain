//! Attachment domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::attachment;

/// Attachment for a paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: i64,
    pub paper_id: i64,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttachment {
    pub paper_id: i64,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
}

impl Attachment {
    pub fn new(paper_id: i64, file_name: Option<String>, file_type: Option<String>, file_size: Option<i64>) -> Self {
        Self {
            id: 0,
            paper_id,
            file_name,
            file_type,
            file_size,
            created_at: Utc::now(),
        }
    }
}

impl From<attachment::Model> for Attachment {
    fn from(model: attachment::Model) -> Self {
        Self {
            id: model.id,
            paper_id: model.paper_id,
            file_name: model.file_name,
            file_type: model.file_type,
            file_size: model.file_size,
            created_at: model.created_at,
        }
    }
}
