//! Keyword repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{keyword, paper_keyword};
use crate::models::{CreateKeyword, Keyword};
use crate::sys::error::{AppError, Result};

/// Repository for Keyword operations
pub struct KeywordRepository;

impl KeywordRepository {
    /// Find all keywords
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Keyword>> {
        let keywords = keyword::Entity::find()
            .order_by_asc(keyword::Column::Word)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query keywords: {}", e)))?;

        info!("Found {} keywords", keywords.len());
        Ok(keywords.into_iter().map(Keyword::from).collect())
    }

    /// Find keyword by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Keyword>> {
        let keyword = keyword::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get keyword: {}", e)))?;

        Ok(keyword.map(Keyword::from))
    }

    /// Find keyword by word
    pub async fn find_by_word(db: &DatabaseConnection, word: &str) -> Result<Option<Keyword>> {
        let keyword = keyword::Entity::find()
            .filter(keyword::Column::Word.eq(word))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query keyword by word: {}", e)))?;

        Ok(keyword.map(Keyword::from))
    }

    /// Create a new keyword
    pub async fn create(db: &DatabaseConnection, create: CreateKeyword) -> Result<Keyword> {
        // Check if keyword already exists
        if Self::find_by_word(db, &create.word).await?.is_some() {
            return Err(AppError::validation(
                "word",
                format!("Keyword '{}' already exists", create.word),
            ));
        }

        let new_keyword = keyword::ActiveModel {
            word: Set(create.word),
            ..Default::default()
        };

        let result = new_keyword
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create keyword: {}", e)))?;

        Ok(Keyword::from(result))
    }

    /// Create or find existing keyword
    pub async fn create_or_find(db: &DatabaseConnection, word: &str) -> Result<Keyword> {
        // Try to find existing keyword
        if let Some(keyword) = Self::find_by_word(db, word).await? {
            return Ok(keyword);
        }

        // Create new keyword
        Self::create(
            db,
            CreateKeyword {
                word: word.to_string(),
            },
        )
        .await
    }

    /// Get keywords for a paper
    pub async fn get_paper_keywords(db: &DatabaseConnection, paper_id: i64) -> Result<Vec<Keyword>> {
        // First get paper_keyword relations
        let relations = paper_keyword::Entity::find()
            .filter(paper_keyword::Column::PaperId.eq(paper_id))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper-keyword relations: {}", e)))?;

        let keyword_ids: Vec<i64> = relations.iter().map(|r| r.keyword_id).collect();

        if keyword_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Then get keywords by IDs
        let keywords = keyword::Entity::find()
            .filter(keyword::Column::Id.is_in(keyword_ids))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper keywords: {}", e)))?;

        Ok(keywords.into_iter().map(Keyword::from).collect())
    }
}
