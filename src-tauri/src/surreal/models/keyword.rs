//! Keyword model for SurrealDB

use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Keyword record representing a paper keyword
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Keyword {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub word: String,
}

/// DTO for creating a new keyword
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateKeyword {
    pub word: String,
}

impl Keyword {
    pub fn new(word: String) -> Self {
        Self { id: None, word }
    }
}

impl From<CreateKeyword> for Keyword {
    fn from(create: CreateKeyword) -> Self {
        Self::new(create.word)
    }
}
