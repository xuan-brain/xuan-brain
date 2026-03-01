//! Paper domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::paper;

/// Paper record representing a research paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: i64,
    pub title: String,
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
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub authors: Vec<AuthorWithOrder>,
}

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

/// Author with order information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorWithOrder {
    pub id: i64,
    pub name: String,
    pub affiliation: Option<String>,
    pub email: Option<String>,
    pub author_order: i32,
    pub is_corresponding: bool,
}

/// Label for papers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub document_count: i32,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaper {
    pub title: String,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdatePaper {
    pub title: Option<String>,
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
            id: 0,
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
            attachments: Vec::new(),
            labels: Vec::new(),
            authors: Vec::new(),
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
            id: 0,
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
            attachments: Vec::new(),
            labels: Vec::new(),
            authors: Vec::new(),
        }
    }
}

// Conversion from database entity to domain model
impl From<paper::Model> for Paper {
    fn from(model: paper::Model) -> Self {
        Self {
            id: model.id,
            title: model.title,
            abstract_text: model.abstract_text,
            doi: model.doi,
            publication_year: model.publication_year,
            publication_date: model.publication_date,
            journal_name: model.journal_name,
            conference_name: model.conference_name,
            volume: model.volume,
            issue: model.issue,
            pages: model.pages,
            url: model.url,
            citation_count: model.citation_count,
            read_status: model.read_status,
            notes: model.notes,
            attachment_path: model.attachment_path,
            created_at: model.created_at,
            updated_at: model.updated_at,
            deleted_at: model.deleted_at,
            attachments: Vec::new(),
            labels: Vec::new(),
            authors: Vec::new(),
        }
    }
}
