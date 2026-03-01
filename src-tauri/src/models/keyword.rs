//! Keyword domain model

use serde::{Deserialize, Serialize};

use crate::database::entities::keyword;

/// Keyword record representing a research keyword
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub id: i64,
    pub word: String,
}

/// DTO for creating a new keyword
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyword {
    pub word: String,
}

impl Keyword {
    pub fn new(word: String) -> Self {
        Self { id: 0, word }
    }
}

impl From<keyword::Model> for Keyword {
    fn from(model: keyword::Model) -> Self {
        Self { id: model.id, word: model.word }
    }
}
