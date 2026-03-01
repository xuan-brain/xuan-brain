//! Clipping repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{clipping, comment};
use crate::models::{Clipping, Comment, CreateClipping, UpdateClipping};
use crate::sys::error::{AppError, Result};

/// Repository for Clipping operations
pub struct ClippingRepository;

impl ClippingRepository {
    /// Create a new clipping (alias for create)
    pub async fn create_clipping(db: &DatabaseConnection, create: CreateClipping) -> Result<Clipping> {
        Self::create(db, create).await
    }

    /// Create a new clipping
    pub async fn create(db: &DatabaseConnection, create: CreateClipping) -> Result<Clipping> {
        let now = chrono::Utc::now();
        let tags_json = if create.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&create.tags).unwrap_or_default())
        };
        let image_paths_json = if create.image_paths.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&create.image_paths).unwrap_or_default())
        };

        let new_clipping = clipping::ActiveModel {
            title: Set(create.title),
            url: Set(create.url),
            content: Set(create.content),
            source_domain: Set(create.source_domain),
            author: Set(create.author),
            published_date: Set(create.published_date),
            excerpt: Set(create.excerpt),
            thumbnail_url: Set(create.thumbnail_url),
            read_status: Set(0),
            notes: Set(None),
            tags: Set(tags_json),
            image_paths: Set(image_paths_json),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = new_clipping
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create clipping: {}", e)))?;

        Ok(Clipping::from(result))
    }

    /// Get all clippings (alias for find_all)
    pub async fn get_all_clippings(db: &DatabaseConnection) -> Result<Vec<Clipping>> {
        Self::find_all(db).await
    }

    /// Get all clippings
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Clipping>> {
        let clippings = clipping::Entity::find()
            .order_by_desc(clipping::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query clippings: {}", e)))?;

        info!("Found {} clippings", clippings.len());

        // Load comments for each clipping
        let mut result = Vec::new();
        for c in clippings {
            let mut clipping = Clipping::from(c);
            clipping.comments = Self::find_comments(db, clipping.id).await?;
            result.push(clipping);
        }

        Ok(result)
    }

    /// Get clipping by ID (alias for find_by_id)
    pub async fn get_clipping_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Clipping>> {
        Self::find_by_id(db, id).await
    }

    /// Get clipping by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Clipping>> {
        let clipping = clipping::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get clipping: {}", e)))?;

        if let Some(c) = clipping {
            let mut clipping = Clipping::from(c);
            clipping.comments = Self::find_comments(db, clipping.id).await?;
            Ok(Some(clipping))
        } else {
            Ok(None)
        }
    }

    /// Get clipping by URL
    pub async fn find_by_url(db: &DatabaseConnection, url: &str) -> Result<Option<Clipping>> {
        let clipping = clipping::Entity::find()
            .filter(clipping::Column::Url.eq(url))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query clipping by URL: {}", e)))?;

        if let Some(c) = clipping {
            let mut clipping = Clipping::from(c);
            clipping.comments = Self::find_comments(db, clipping.id).await?;
            Ok(Some(clipping))
        } else {
            Ok(None)
        }
    }

    /// Update clipping (alias for update)
    pub async fn update_clipping(
        db: &DatabaseConnection,
        id: i64,
        update: UpdateClipping,
    ) -> Result<Option<Clipping>> {
        Self::update(db, id, update).await
    }

    /// Update clipping
    pub async fn update(
        db: &DatabaseConnection,
        id: i64,
        update: UpdateClipping,
    ) -> Result<Option<Clipping>> {
        let clipping = clipping::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find clipping: {}", e)))?;

        let Some(c) = clipping else {
            return Ok(None);
        };

        let mut clipping: clipping::ActiveModel = c.into();

        if let Some(title) = update.title {
            clipping.title = Set(title);
        }
        if let Some(url) = update.url {
            clipping.url = Set(url);
        }
        if let Some(content) = update.content {
            clipping.content = Set(Some(content));
        }
        if let Some(source_domain) = update.source_domain {
            clipping.source_domain = Set(Some(source_domain));
        }
        if let Some(author) = update.author {
            clipping.author = Set(Some(author));
        }
        if let Some(published_date) = update.published_date {
            clipping.published_date = Set(Some(published_date));
        }
        if let Some(excerpt) = update.excerpt {
            clipping.excerpt = Set(Some(excerpt));
        }
        if let Some(thumbnail_url) = update.thumbnail_url {
            clipping.thumbnail_url = Set(Some(thumbnail_url));
        }
        if let Some(read_status) = update.read_status {
            clipping.read_status = Set(read_status);
        }
        if let Some(notes) = update.notes {
            clipping.notes = Set(Some(notes));
        }
        if let Some(tags) = update.tags {
            let tags_json = if tags.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&tags).unwrap_or_default())
            };
            clipping.tags = Set(tags_json);
        }
        if let Some(image_paths) = update.image_paths {
            let image_paths_json = if image_paths.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&image_paths).unwrap_or_default())
            };
            clipping.image_paths = Set(image_paths_json);
        }

        clipping.updated_at = Set(chrono::Utc::now());

        let result = clipping
            .update(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to update clipping: {}", e)))?;

        let mut clipping = Clipping::from(result);
        clipping.comments = Self::find_comments(db, clipping.id).await?;
        Ok(Some(clipping))
    }

    // ==================== Comment operations ====================

    /// Get comments for a clipping (public method)
    pub async fn get_comments(db: &DatabaseConnection, clipping_id: i64) -> Result<Vec<Comment>> {
        Self::find_comments(db, clipping_id).await
    }

    /// Find comments for a clipping
    async fn find_comments(db: &DatabaseConnection, clipping_id: i64) -> Result<Vec<Comment>> {
        let comments = comment::Entity::find()
            .filter(comment::Column::ClippingId.eq(clipping_id))
            .order_by_asc(comment::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get comments: {}", e)))?;

        Ok(comments.into_iter().map(Comment::from).collect())
    }

    /// Add a comment to a clipping
    pub async fn add_comment(
        db: &DatabaseConnection,
        clipping_id: i64,
        content: &str,
    ) -> Result<Comment> {
        // Verify clipping exists
        let clipping_exists = clipping::Entity::find_by_id(clipping_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find clipping: {}", e)))?
            .is_some();

        if !clipping_exists {
            return Err(AppError::not_found("Clipping", clipping_id.to_string()));
        }

        let now = chrono::Utc::now();
        let new_comment = comment::ActiveModel {
            clipping_id: Set(clipping_id),
            content: Set(content.to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = new_comment
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add comment: {}", e)))?;

        // Update clipping's updated_at
        Self::touch_clipping(db, clipping_id).await?;

        Ok(Comment::from(result))
    }

    /// Update a comment
    pub async fn update_comment(
        db: &DatabaseConnection,
        comment_id: i64,
        content: &str,
    ) -> Result<Comment> {
        let comment = comment::Entity::find_by_id(comment_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find comment: {}", e)))?
            .ok_or_else(|| AppError::not_found("Comment", comment_id.to_string()))?;

        let clipping_id = comment.clipping_id;

        let mut comment: comment::ActiveModel = comment.into();
        comment.content = Set(content.to_string());
        comment.updated_at = Set(chrono::Utc::now());

        let result = comment
            .update(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to update comment: {}", e)))?;

        // Update clipping's updated_at
        Self::touch_clipping(db, clipping_id).await?;

        Ok(Comment::from(result))
    }

    /// Delete a comment
    pub async fn delete_comment(db: &DatabaseConnection, comment_id: i64) -> Result<()> {
        let comment = comment::Entity::find_by_id(comment_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find comment: {}", e)))?
            .ok_or_else(|| AppError::not_found("Comment", comment_id.to_string()))?;

        let clipping_id = comment.clipping_id;

        comment::Entity::delete_by_id(comment_id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete comment: {}", e)))?;

        // Update clipping's updated_at
        Self::touch_clipping(db, clipping_id).await?;

        Ok(())
    }

    /// Update clipping's updated_at timestamp
    async fn touch_clipping(db: &DatabaseConnection, clipping_id: i64) -> Result<()> {
        let clipping = clipping::Entity::find_by_id(clipping_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find clipping: {}", e)))?;

        if let Some(clipping) = clipping {
            let mut clipping: clipping::ActiveModel = clipping.into();
            clipping.updated_at = Set(chrono::Utc::now());
            clipping.update(db).await.map_err(|e| {
                AppError::generic(format!("Failed to update clipping timestamp: {}", e))
            })?;
        }

        Ok(())
    }
}
