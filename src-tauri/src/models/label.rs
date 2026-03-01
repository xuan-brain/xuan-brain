//! Label domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::label;

/// Label record representing a paper label/tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub document_count: i32,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLabel {
    pub name: String,
    #[serde(default = "default_color")]
    pub color: String,
}

/// DTO for updating a label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLabel {
    pub name: Option<String>,
    pub color: Option<String>,
}

fn default_color() -> String {
    "#1976D2".to_string()
}

impl Label {
    pub fn new(name: String, color: Option<String>) -> Self {
        Self {
            id: 0,
            name,
            color: color.unwrap_or_else(default_color),
            document_count: 0,
            created_at: Utc::now(),
        }
    }
}

impl From<CreateLabel> for Label {
    fn from(create: CreateLabel) -> Self {
        Self::new(create.name, Some(create.color))
    }
}

impl From<label::Model> for Label {
    fn from(model: label::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            color: model.color,
            document_count: model.document_count,
            created_at: model.created_at,
        }
    }
}
