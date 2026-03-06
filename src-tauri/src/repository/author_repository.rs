//! Author repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{author, paper_author};
use crate::models::{Author, AuthorNameParser, AuthorNameParts, CreateAuthor};
use crate::sys::error::{AppError, Result};

/// Repository for Author operations
pub struct AuthorRepository;

impl AuthorRepository {
    /// Find all authors
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Author>> {
        let authors = author::Entity::find()
            .order_by_asc(author::Column::FirstName)
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
            first_name: Set(create.first_name),
            last_name: Set(create.last_name),
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

    /// Create or find existing author by full name and email
    /// This method parses the full name and is used for sources that only provide full names (e.g., arXiv)
    pub async fn create_or_find(
        db: &DatabaseConnection,
        full_name: &str,
        email: Option<&str>,
    ) -> Result<Author> {
        let name_parts = AuthorNameParser::parse(full_name);
        Self::create_or_find_by_parts(db, &name_parts, email).await
    }

    /// Create or find existing author by structured name parts
    /// This is used for sources that provide given/family names separately (e.g., DOI, PubMed)
    pub async fn create_or_find_from_parts(
        db: &DatabaseConnection,
        given_name: Option<&str>,
        family_name: Option<&str>,
        email: Option<&str>,
    ) -> Result<Author> {
        let name_parts = AuthorNameParser::from_parts(given_name, family_name);
        Self::create_or_find_by_parts(db, &name_parts, email).await
    }

    /// Internal method to create or find by name parts
    async fn create_or_find_by_parts(
        db: &DatabaseConnection,
        name_parts: &AuthorNameParts,
        email: Option<&str>,
    ) -> Result<Author> {
        // Skip if first_name is empty
        if name_parts.first_name.is_empty() {
            return Err(AppError::generic("Author first_name cannot be empty"));
        }

        // Build query based on whether last_name and email exist
        let mut query = author::Entity::find()
            .filter(author::Column::FirstName.eq(&name_parts.first_name));

        // Handle last_name (can be None or Some)
        match &name_parts.last_name {
            Some(last) if !last.is_empty() => {
                query = query.filter(author::Column::LastName.eq(last));
            }
            _ => {
                query = query.filter(author::Column::LastName.is_null());
            }
        }

        // Handle email
        match email {
            Some(email_val) => {
                query = query.filter(author::Column::Email.eq(email_val));
            }
            None => {
                query = query.filter(author::Column::Email.is_null());
            }
        }

        let existing = query
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query author: {}", e)))?;

        if let Some(author) = existing {
            return Ok(Author::from(author));
        }

        // Create new author
        Self::create(
            db,
            CreateAuthor {
                first_name: name_parts.first_name.clone(),
                last_name: name_parts.last_name.clone(),
                affiliation: None,
                email: email.map(|s| s.to_string()),
            },
        )
        .await
    }

    /// Get authors for a paper, ordered by author_order
    pub async fn get_paper_authors(db: &DatabaseConnection, paper_id: i64) -> Result<Vec<Author>> {
        // First get paper_author relations
        let relations = paper_author::Entity::find()
            .filter(paper_author::Column::PaperId.eq(paper_id))
            .order_by_asc(paper_author::Column::AuthorOrder)
            .all(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to get paper-author relations: {}", e))
            })?;

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
