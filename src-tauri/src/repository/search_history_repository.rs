//! Search history repository for SQLite using SeaORM
//!
//! Manages user search query history for quick access to past searches.

use sea_orm::*;
use tracing::info;

use crate::database::entities::search_history;
use crate::sys::error::{AppError, Result};

/// Repository for search history operations
pub struct SearchHistoryRepository;

impl SearchHistoryRepository {
    /// Add a search query to history
    /// Returns the created search history entry
    pub async fn add(db: &DatabaseConnection, query: &str) -> Result<search_history::Model> {
        let now = chrono::Utc::now();
        let new_history = search_history::ActiveModel {
            query: Set(query.to_string()),
            created_at: Set(now),
            ..Default::default()
        };

        let result = new_history
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add search history: {}", e)))?;

        info!("Added search history: '{}'", query);
        Ok(result)
    }

    /// Get recent search history
    /// Returns search history entries ordered by creation time (newest first)
    pub async fn get_recent(db: &DatabaseConnection, limit: u64) -> Result<Vec<search_history::Model>> {
        let history = search_history::Entity::find()
            .order_by_desc(search_history::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get search history: {}", e)))?;

        info!("Found {} search history entries", history.len());
        Ok(history)
    }

    /// Clear all search history
    pub async fn clear(db: &DatabaseConnection) -> Result<()> {
        search_history::Entity::delete_many()
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to clear search history: {}", e)))?;

        info!("Cleared all search history");
        Ok(())
    }

    /// Delete a specific search history entry by ID
    pub async fn delete_by_id(db: &DatabaseConnection, id: i64) -> Result<()> {
        search_history::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete search history: {}", e)))?;

        info!("Deleted search history entry with id: {}", id);
        Ok(())
    }
}
