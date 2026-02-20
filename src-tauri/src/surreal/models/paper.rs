//! Paper model for SurrealDB

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Paper record representing a research paper
/// Note: `abstract` is renamed to `abstract_text` because `abstract` is a Rust keyword
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Paper {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub publication_date: Option<String>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub citation_count: i32,
    pub read_status: String,
    pub notes: Option<String>,
    pub attachment_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// DTO for creating a new paper
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreatePaper {
    pub title: String,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub publication_date: Option<String>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub attachment_path: Option<String>,
}

/// DTO for updating paper details
#[derive(Debug, Clone, Serialize, Deserialize, Default, SurrealValue)]
pub struct UpdatePaper {
    pub title: Option<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub publication_date: Option<String>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub read_status: Option<String>,
    pub notes: Option<String>,
    pub attachment_path: Option<String>,
}

impl Paper {
    /// Create a new paper with default values
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            abstract_text: None,
            doi: None,
            publication_year: None,
            publication_date: None,
            journal_name: None,
            conference_name: None,
            volume: None,
            issue: None,
            pages: None,
            url: None,
            citation_count: 0,
            read_status: "unread".to_string(),
            notes: None,
            attachment_path: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Check if paper is deleted (soft delete)
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

impl From<CreatePaper> for Paper {
    fn from(create: CreatePaper) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title: create.title,
            abstract_text: create.abstract_text,
            doi: create.doi,
            publication_year: create.publication_year,
            publication_date: create.publication_date,
            journal_name: create.journal_name,
            conference_name: create.conference_name,
            volume: create.volume,
            issue: create.issue,
            pages: create.pages,
            url: create.url,
            citation_count: 0,
            read_status: "unread".to_string(),
            notes: None,
            attachment_path: create.attachment_path,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}
