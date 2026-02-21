//! Keyword repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateKeyword, Keyword};
use crate::sys::error::{AppError, Result};
use tracing::info;

/// Repository for Keyword operations
pub struct KeywordRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> KeywordRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Find all keywords
    pub async fn find_all(&self) -> Result<Vec<Keyword>> {
        let result: Vec<Keyword> = self
            .db
            .query("SELECT * FROM keyword ORDER BY word")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query keywords: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} keywords", result.len());
        Ok(result)
    }

    /// Find keyword by ID (string format like "keyword:123")
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Keyword>> {
        let id = id.to_string();
        let result: Vec<Keyword> = self
            .db
            .query("SELECT * FROM type::record($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get keyword: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Find keyword by word
    pub async fn find_by_word(&self, word: &str) -> Result<Option<Keyword>> {
        let word = word.to_string();
        let result: Vec<Keyword> = self
            .db
            .query("SELECT * FROM keyword WHERE word = $word LIMIT 1")
            .bind(("word", word))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query keyword by word: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new keyword
    pub async fn create(&self, keyword: CreateKeyword) -> Result<Keyword> {
        // Check if keyword already exists
        if (self.find_by_word(&keyword.word).await?).is_some() {
            return Err(AppError::validation(
                "word",
                format!("Keyword '{}' already exists", keyword.word),
            ));
        }

        let keyword = Keyword::from(keyword);

        let result: Vec<Keyword> = self
            .db
            .query("CREATE keyword CONTENT $keyword")
            .bind(("keyword", keyword))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create keyword: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create keyword".to_string()))
    }

    /// Create or find existing keyword
    pub async fn create_or_find(&self, word: &str) -> Result<Keyword> {
        // Try to find existing keyword
        if let Some(keyword) = self.find_by_word(word).await? {
            return Ok(keyword);
        }

        // Create new keyword
        self.create(CreateKeyword {
            word: word.to_string(),
        })
        .await
    }

    /// Get keywords for a paper
    pub async fn get_paper_keywords(&self, paper_id: &str) -> Result<Vec<Keyword>> {
        let paper_id = paper_id.to_string();
        let result: Vec<Keyword> = self
            .db
            .query(
                r#"
                SELECT * FROM keyword
                WHERE id IN (SELECT VALUE `out` FROM paper_keyword WHERE `in` = type::record($paper))
                "#,
            )
            .bind(("paper", paper_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper keywords: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }
}
