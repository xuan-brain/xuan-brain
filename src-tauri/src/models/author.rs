//! Author domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::author;

/// Author record representing a research paper author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuthor {
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
}

impl Author {
    pub fn new(name: String) -> Self {
        Self {
            id: 0,
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
            id: 0,
            name: create.name,
            affiliation: create.affiliation,
            email: create.email,
            created_at: Utc::now(),
        }
    }
}

impl From<author::Model> for Author {
    fn from(model: author::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            affiliation: model.affiliation,
            email: model.email,
            created_at: model.created_at,
        }
    }
}
