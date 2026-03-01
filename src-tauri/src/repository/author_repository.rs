//! Author repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{author, paper_author};
use crate::models::{Author, CreateAuthor};
use crate::sys::error::{AppError, Result};

/// Repository for Author operations
pub struct AuthorRepository;

impl AuthorRepository {
    /// Find all authors
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Author>> {
        let authors = author::Entity::find()
            .order_by_asc(author::Column::Name)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query authors: {}", e)))?;

        info!("Found {} authors", authors.len());
        Ok(authors.into_iter().map(Author::from).collect())
    }

    /// Find author by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Author>> {
        let author = author::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get author: {}", e)))?;

        Ok(author.map(Author::from))
    }

    /// Create a new author
    pub async fn create(db: &DatabaseConnection, create: CreateAuthor) -> Result<Author> {
        let now = chrono::Utc::now();
        let new_author = author::ActiveModel {
            name: Set(create.name),
            affiliation: Set(create.affiliation),
            email: Set(create.email),
            created_at: Set(now),
            ..Default::default()
        };

        let result = new_author
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create author: {}", e)))?;

        Ok(Author::from(result))
    }

    /// Create or find existing author by name and email
    pub async fn create_or_find(
        db: &DatabaseConnection,
        name: &str,
        email: Option<&str>,
    ) -> Result<Author> {
        // Try to find existing author
        let existing = if let Some(email_val) = email {
            author::Entity::find()
                .filter(author::Column::Name.eq(name))
                .filter(author::Column::Email.eq(email_val))
                .one(db)
                .await
        } else {
            author::Entity::find()
                .filter(author::Column::Name.eq(name))
                .filter(author::Column::Email.is_null())
                .one(db)
                .await
        }
        .map_err(|e| AppError::generic(format!("Failed to query author: {}", e)))?;

        if let Some(author) = existing {
            return Ok(Author::from(author));
        }

        // Create new author
        Self::create(
            db,
            CreateAuthor {
                name: name.to_string(),
                affiliation: None,
                email: email.map(|s| s.to_string()),
            },
        )
        .await
    }

    /// Get authors for a paper
    pub async fn get_paper_authors(db: &DatabaseConnection, paper_id: i64) -> Result<Vec<Author>> {
        // First get paper_author relations
        let relations = paper_author::Entity::find()
            .filter(paper_author::Column::PaperId.eq(paper_id))
            .order_by_asc(paper_author::Column::AuthorOrder)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper-author relations: {}", e)))?;

        let author_ids: Vec<i64> = relations.iter().map(|r| r.author_id).collect();

        if author_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Then get authors by IDs
        let authors = author::Entity::find()
            .filter(author::Column::Id.is_in(author_ids))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper authors: {}", e)))?;

        // Sort by author_order from relations
        let author_map: std::collections::HashMap<i64, Author> = authors
            .into_iter()
            .map(|a| (a.id, Author::from(a)))
            .collect();

        let result: Vec<Author> = relations
            .into_iter()
            .filter_map(|r| author_map.get(&r.author_id).cloned())
            .collect();

        Ok(result)
    }
}
