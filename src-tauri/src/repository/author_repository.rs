//! Author repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{Author, CreateAuthor};
use crate::sys::error::{AppError, Result};
use tracing::info;

/// Repository for Author operations
pub struct AuthorRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> AuthorRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Find all authors
    pub async fn find_all(&self) -> Result<Vec<Author>> {
        let result: Vec<Author> = self
            .db
            .query("SELECT * FROM author ORDER BY name")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query authors: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} authors", result.len());
        Ok(result)
    }

    /// Find author by ID (string format like "author:123")
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Author>> {
        let id = id.to_string();
        let result: Vec<Author> = self
            .db
            .query("SELECT * FROM type::thing($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get author: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new author
    pub async fn create(&self, author: CreateAuthor) -> Result<Author> {
        let author = Author::from(author);

        let result: Vec<Author> = self
            .db
            .query("CREATE author CONTENT $author")
            .bind(("author", author))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create author: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create author".to_string()))
    }

    /// Create or find existing author by name and email
    pub async fn create_or_find(&self, name: &str, email: Option<&str>) -> Result<Author> {
        let name = name.to_string();
        let email = email.map(|s| s.to_string());

        // Try to find existing author
        let result: Vec<Author> = if let Some(ref email_val) = email {
            self.db
                .query("SELECT * FROM author WHERE name = $name AND email = $email LIMIT 1")
                .bind(("name", name.clone()))
                .bind(("email", email_val.clone()))
                .await
                .map_err(|e| AppError::generic(format!("Failed to query author: {}", e)))?
                .take(0)
                .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?
        } else {
            self.db
                .query("SELECT * FROM author WHERE name = $name AND email IS NONE LIMIT 1")
                .bind(("name", name.clone()))
                .await
                .map_err(|e| AppError::generic(format!("Failed to query author: {}", e)))?
                .take(0)
                .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?
        };

        if let Some(author) = result.into_iter().next() {
            return Ok(author);
        }

        // Create new author
        self.create(CreateAuthor {
            name,
            affiliation: None,
            email,
        })
        .await
    }

    /// Get authors for a paper
    pub async fn get_paper_authors(&self, paper_id: &str) -> Result<Vec<Author>> {
        let paper_id = paper_id.to_string();
        let result: Vec<Author> = self
            .db
            .query(
                r#"
                SELECT * FROM author
                WHERE id IN (SELECT VALUE `out` FROM paper_author WHERE `in` = type::thing($paper))
                ORDER BY (SELECT VALUE author_order FROM paper_author WHERE `in` = type::thing($paper) AND `out` = author.id)[0]
                "#,
            )
            .bind(("paper", paper_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper authors: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }
}
