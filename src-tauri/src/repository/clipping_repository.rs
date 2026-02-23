//! Clipping repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{Clipping, CreateClipping, UpdateClipping};
use crate::sys::error::{AppError, Result};
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
        let result: Vec<Clipping> = db
            .query("SELECT * FROM clipping ORDER BY created_at DESC")
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
            .query("SELECT * FROM type::record($id) LIMIT 1")
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
            .await
            .map_err(|e| AppError::generic(format!("Failed to update clipping: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }
}
