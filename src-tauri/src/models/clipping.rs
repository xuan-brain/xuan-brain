//! Clipping domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::clipping;

/// Clipping record representing a web content snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clipping {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: i32,
    pub notes: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub image_paths: Vec<String>,
    #[serde(default)]
    pub comments: Vec<Comment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTO for creating a new clipping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClipping {
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub image_paths: Vec<String>,
}

/// DTO for updating clipping details
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateClipping {
    pub title: Option<String>,
    pub url: Option<String>,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: Option<i32>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub image_paths: Option<Vec<String>>,
}

impl Clipping {
    /// Create a new clipping with default values
    pub fn new(title: String, url: String, content: Option<String>, source_domain: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            title,
            url,
            content,
            source_domain,
            author: None,
            published_date: None,
            excerpt: None,
            thumbnail_url: None,
            read_status: 0,
            notes: None,
            tags: Vec::new(),
            image_paths: Vec::new(),
            comments: Vec::new(),
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
            id: 0,
            title: create.title,
            url: create.url,
            content: create.content,
            source_domain: create.source_domain,
            author: create.author,
            published_date: create.published_date,
            excerpt: create.excerpt,
            thumbnail_url: create.thumbnail_url,
            read_status: 0,
            notes: None,
            tags: create.tags,
            image_paths: create.image_paths,
            comments: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl From<clipping::Model> for Clipping {
    fn from(model: clipping::Model) -> Self {
        // Parse JSON arrays
        let tags = model
            .tags
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        let image_paths = model
            .image_paths
            .and_then(|p| serde_json::from_str(&p).ok())
            .unwrap_or_default();

        Self {
            id: model.id,
            title: model.title,
            url: model.url,
            content: model.content,
            source_domain: model.source_domain,
            author: model.author,
            published_date: model.published_date,
            excerpt: model.excerpt,
            thumbnail_url: model.thumbnail_url,
            read_status: model.read_status,
            notes: model.notes,
            tags,
            image_paths,
            comments: Vec::new(),
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

// Import Comment from the comment module
use super::comment::Comment;
