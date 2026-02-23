//! Author model for SurrealDB

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Author record representing a research paper author
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Author {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new author
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateAuthor {
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
}

impl Author {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name,
            affiliation: None,
            email: None,
            created_at: Utc::now(),
        }
    }
}

impl From<CreateAuthor> for Author {
    fn from(create: CreateAuthor) -> Self {
        Self {
            id: None,
            name: create.name,
            affiliation: create.affiliation,
            email: create.email,
            created_at: Utc::now(),
        }
    }
}
