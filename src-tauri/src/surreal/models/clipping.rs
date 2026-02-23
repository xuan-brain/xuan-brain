//! Clipping model for web content snippets

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue};

/// Clipping record representing a web content snippet
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Clipping {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    pub author: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: i32,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub image_paths: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new clipping
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct CreateClipping {
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    pub author: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags: Vec<String>,
    pub image_paths: Vec<String>,
}

/// DTO for updating clipping details
#[derive(Debug, Clone, Serialize, Deserialize, Default, SurrealValue)]
pub struct UpdateClipping {
    pub title: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<DateTime<Utc>>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: Option<i32>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub image_paths: Option<Vec<String>>,
}

impl Clipping {
    /// Create a new clipping with default values
    pub fn new(title: String, url: String, content: String, source_domain: String) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title,
            url,
            content,
            source_domain,
            author: None,
            published_date: None,
            excerpt: None,
            thumbnail_url: None,
            read_status: 0, // 0 = unread
            notes: None,
            tags: Vec::new(),
            image_paths: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if clipping is read
    pub fn is_read(&self) -> bool {
        self.read_status > 0
    }
}

impl From<CreateClipping> for Clipping {
    fn from(create: CreateClipping) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            title: create.title,
            url: create.url,
            content: create.content,
            source_domain: create.source_domain,
            author: create.author,
            published_date: create.published_date,
            excerpt: create.excerpt,
            thumbnail_url: create.thumbnail_url,
            read_status: 0, // 0 = unread
            notes: None,
            tags: create.tags,
            image_paths: create.image_paths,
            created_at: now,
            updated_at: now,
        }
    }
}
