//! Data Transfer Objects for clip commands

use serde::{Deserialize, Serialize};

/// Comment DTO for clip comments
#[derive(Serialize, Clone)]
pub struct CommentDto {
    pub id: String,
    pub clipping_id: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Response DTO for clip list and detail views
#[derive(Serialize, Clone)]
pub struct ClipDto {
    pub id: String,
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
    pub tags: Vec<String>,
    pub image_paths: Vec<String>,
    pub comments: Vec<CommentDto>,
    pub created_at: String,
    pub updated_at: String,
}

/// Request DTO for creating a new clip
#[derive(Deserialize, Debug)]
pub struct CreateClipRequest {
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Response DTO for create operation
#[derive(Serialize)]
pub struct CreateClipResponse {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub image_paths: Vec<String>,
}
