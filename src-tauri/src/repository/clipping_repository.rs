//! Clipping repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{Clipping, Comment, CreateClipping, UpdateClipping};
use crate::sys::error::{AppError, Result};
use chrono::Utc;
use surrealdb_types::uuid::Uuid;
use tracing::info;

/// Repository for Clipping operations
pub struct ClippingRepository;

impl ClippingRepository {
    /// Create a new clipping
    pub async fn create_clipping(db: &SurrealClient, clipping: CreateClipping) -> Result<Clipping> {
        let clipping = Clipping::from(clipping);
        let result: Vec<Clipping> = db
            .query("CREATE clipping CONTENT $clipping")
            .bind(("clipping", clipping))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create clipping: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create clipping".to_string()))
    }

    /// Get all clippings
    pub async fn get_all_clippings(db: &SurrealClient) -> Result<Vec<Clipping>> {
        // Use IF/ELSE to handle NONE comments field
        let result: Vec<Clipping> = db
            .query("SELECT *, IF comments IS NONE THEN [] ELSE comments END AS comments FROM clipping ORDER BY created_at DESC")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query clippings: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} clippings", result.len());
        Ok(result)
    }

    /// Get clipping by ID
    pub async fn get_clipping_by_id(
        db: &SurrealClient,
        id: &str,
    ) -> Result<Option<Clipping>> {
        // Ensure ID has the 'clipping:' prefix for type::record()
        let record_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("clipping:{}", id)
        };
        let result: Vec<Clipping> = db
            .query("SELECT *, IF comments IS NONE THEN [] ELSE comments END AS comments FROM type::record($id) LIMIT 1")
            .bind(("id", record_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get clipping: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Get clipping by URL
    pub async fn get_clipping_by_url(
        db: &SurrealClient,
        url: &str,
    ) -> Result<Option<Clipping>> {
        let url = url.to_string();
        let result: Vec<Clipping> = db
            .query("SELECT * FROM clipping WHERE url = $url LIMIT 1")
            .bind(("url", url))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query clipping by URL: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Update clipping
    pub async fn update_clipping(
        db: &SurrealClient,
        id: &str,
        clipping: UpdateClipping,
    ) -> Result<Option<Clipping>> {
        // Ensure ID has the 'clipping:' prefix for type::record()
        let record_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("clipping:{}", id)
        };

        // Build update query dynamically based on provided fields
        let mut sets = Vec::new();
        if clipping.title.is_some() {
            sets.push("title = $title");
        }
        if clipping.url.is_some() {
            sets.push("url = $url");
        }
        if clipping.content.is_some() {
            sets.push("content = $content");
        }
        if clipping.source_domain.is_some() {
            sets.push("source_domain = $source_domain");
        }
        if clipping.author.is_some() {
            sets.push("author = $author");
        }
        if clipping.published_date.is_some() {
            sets.push("published_date = $published_date");
        }
        if clipping.excerpt.is_some() {
            sets.push("excerpt = $excerpt");
        }
        if clipping.thumbnail_url.is_some() {
            sets.push("thumbnail_url = $thumbnail_url");
        }
        if clipping.read_status.is_some() {
            sets.push("read_status = $read_status");
        }
        if clipping.notes.is_some() {
            sets.push("notes = $notes");
        }
        if clipping.tags.is_some() {
            sets.push("tags = $tags");
        }
        if clipping.image_paths.is_some() {
            sets.push("image_paths = $image_paths");
        }
        if clipping.comments.is_some() {
            sets.push("comments = $comments");
        }

        // Always update updated_at
        sets.push("updated_at = time::now()");

        let query = format!("UPDATE type::record($id) SET {}", sets.join(", "));

        let result: Vec<Clipping> = db
            .query(&query)
            .bind(("id", record_id.clone()))
            .bind(("title", clipping.title))
            .bind(("url", clipping.url))
            .bind(("content", clipping.content))
            .bind(("source_domain", clipping.source_domain))
            .bind(("author", clipping.author))
            .bind(("published_date", clipping.published_date))
            .bind(("excerpt", clipping.excerpt))
            .bind(("thumbnail_url", clipping.thumbnail_url))
            .bind(("read_status", clipping.read_status))
            .bind(("notes", clipping.notes))
            .bind(("tags", clipping.tags))
            .bind(("image_paths", clipping.image_paths))
            .bind(("comments", clipping.comments))
            .await
            .map_err(|e| AppError::generic(format!("Failed to update clipping: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Add a comment to a clipping
    pub async fn add_comment(
        db: &SurrealClient,
        clip_id: &str,
        content: &str,
    ) -> Result<Clipping> {
        let record_id = if clip_id.contains(':') {
            clip_id.to_string()
        } else {
            format!("clipping:{}", clip_id)
        };

        // Get current clipping to append comment
        let clipping = Self::get_clipping_by_id(db, clip_id)
            .await?
            .ok_or_else(|| AppError::not_found("Clipping", clip_id.to_string()))?;

        let now = Utc::now();
        let new_comment = Comment {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            created_at: now,
            updated_at: now,
        };

        let mut comments = clipping.comments;
        comments.push(new_comment);

        // Update with new comments array
        let update = UpdateClipping {
            comments: Some(comments),
            ..Default::default()
        };

        Self::update_clipping(db, clip_id, update)
            .await?
            .ok_or_else(|| AppError::generic("Failed to add comment".to_string()))
    }

    /// Update a comment in a clipping
    pub async fn update_comment(
        db: &SurrealClient,
        clip_id: &str,
        comment_id: &str,
        content: &str,
    ) -> Result<Clipping> {
        // Get current clipping
        let clipping = Self::get_clipping_by_id(db, clip_id)
            .await?
            .ok_or_else(|| AppError::not_found("Clipping", clip_id.to_string()))?;

        // Find and update the comment
        let mut comments = clipping.comments;
        let mut found = false;
        let now = Utc::now();

        for comment in &mut comments {
            if comment.id == comment_id {
                comment.content = content.to_string();
                comment.updated_at = now;
                found = true;
                break;
            }
        }

        if !found {
            return Err(AppError::not_found("Comment", comment_id.to_string()));
        }

        // Update with modified comments array
        let update = UpdateClipping {
            comments: Some(comments),
            ..Default::default()
        };

        Self::update_clipping(db, clip_id, update)
            .await?
            .ok_or_else(|| AppError::generic("Failed to update comment".to_string()))
    }

    /// Delete a comment from a clipping
    pub async fn delete_comment(
        db: &SurrealClient,
        clip_id: &str,
        comment_id: &str,
    ) -> Result<Clipping> {
        // Get current clipping
        let clipping = Self::get_clipping_by_id(db, clip_id)
            .await?
            .ok_or_else(|| AppError::not_found("Clipping", clip_id.to_string()))?;

        // Remove the comment
        let comments: Vec<Comment> = clipping
            .comments
            .into_iter()
            .filter(|c| c.id != comment_id)
            .collect();

        // Update with modified comments array
        let update = UpdateClipping {
            comments: Some(comments),
            ..Default::default()
        };

        Self::update_clipping(db, clip_id, update)
            .await?
            .ok_or_else(|| AppError::generic("Failed to delete comment".to_string()))
    }
}
