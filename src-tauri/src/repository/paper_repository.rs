//! Paper repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreatePaper, Paper, UpdatePaper};
use crate::sys::error::{AppError, Result};
use tracing::info;

/// Repository for Paper operations
pub struct PaperRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> PaperRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Find all non-deleted papers
    pub async fn find_all(&self) -> Result<Vec<Paper>> {
        let result: Vec<Paper> = self
            .db
            .query("SELECT * FROM paper WHERE deleted_at IS NONE ORDER BY created_at DESC")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query papers: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} papers", result.len());
        Ok(result)
    }

    /// Find all deleted papers (trash)
    pub async fn find_deleted(&self) -> Result<Vec<Paper>> {
        let result: Vec<Paper> = self
            .db
            .query("SELECT * FROM paper WHERE deleted_at IS NOT NONE ORDER BY deleted_at DESC")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query deleted papers: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }

    /// Find paper by ID (string format like "paper:123")
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Paper>> {
        let id = id.to_string();
        let result: Vec<Paper> = self
            .db
            .query("SELECT * FROM type::thing($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Find paper by DOI
    pub async fn find_by_doi(&self, doi: &str) -> Result<Option<Paper>> {
        let doi = doi.to_string();
        let result: Vec<Paper> = self
            .db
            .query("SELECT * FROM paper WHERE doi = $doi LIMIT 1")
            .bind(("doi", doi))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query paper by DOI: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new paper
    pub async fn create(&self, paper: CreatePaper) -> Result<Paper> {
        let paper = Paper::from(paper);
        let result: Vec<Paper> = self
            .db
            .query("CREATE paper CONTENT $paper")
            .bind(("paper", paper))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create paper: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create paper".to_string()))
    }

    /// Update paper
    pub async fn update(&self, id: &str, update: UpdatePaper) -> Result<Paper> {
        let id = id.to_string();

        // Build update query dynamically based on provided fields
        let mut sets = Vec::new();
        if update.title.is_some() {
            sets.push("title = $title");
        }
        if update.abstract_text.is_some() {
            sets.push("`abstract` = $abstract_text");
        }
        if update.doi.is_some() {
            sets.push("doi = $doi");
        }
        if update.publication_year.is_some() {
            sets.push("publication_year = $publication_year");
        }
        if update.publication_date.is_some() {
            sets.push("publication_date = $publication_date");
        }
        if update.journal_name.is_some() {
            sets.push("journal_name = $journal_name");
        }
        if update.conference_name.is_some() {
            sets.push("conference_name = $conference_name");
        }
        if update.volume.is_some() {
            sets.push("volume = $volume");
        }
        if update.issue.is_some() {
            sets.push("issue = $issue");
        }
        if update.pages.is_some() {
            sets.push("pages = $pages");
        }
        if update.url.is_some() {
            sets.push("url = $url");
        }
        if update.read_status.is_some() {
            sets.push("read_status = $read_status");
        }
        if update.notes.is_some() {
            sets.push("notes = $notes");
        }
        if update.attachment_path.is_some() {
            sets.push("attachment_path = $attachment_path");
        }
        sets.push("updated_at = time::now()");

        let query = format!("UPDATE type::thing($id) SET {}", sets.join(", "));

        let result: Vec<Paper> = self
            .db
            .query(&query)
            .bind(("id", id.clone()))
            .bind(("title", update.title))
            .bind(("abstract_text", update.abstract_text))
            .bind(("doi", update.doi))
            .bind(("publication_year", update.publication_year))
            .bind(("publication_date", update.publication_date))
            .bind(("journal_name", update.journal_name))
            .bind(("conference_name", update.conference_name))
            .bind(("volume", update.volume))
            .bind(("issue", update.issue))
            .bind(("pages", update.pages))
            .bind(("url", update.url))
            .bind(("read_status", update.read_status))
            .bind(("notes", update.notes))
            .bind(("attachment_path", update.attachment_path))
            .await
            .map_err(|e| AppError::generic(format!("Failed to update paper: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("Paper", id))
    }

    /// Soft delete paper (move to trash)
    pub async fn soft_delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.db
            .query("UPDATE type::thing($id) SET deleted_at = time::now()")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to soft delete paper: {}", e)))?;

        Ok(())
    }

    /// Restore paper from trash
    pub async fn restore(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.db
            .query("UPDATE type::thing($id) SET deleted_at = NONE")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to restore paper: {}", e)))?;

        Ok(())
    }

    /// Permanently delete paper
    pub async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.db
            .query("DELETE type::thing($id)")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete paper: {}", e)))?;

        Ok(())
    }

    /// Search papers using BM25 full-text search
    pub async fn search(&self, query: &str) -> Result<Vec<Paper>> {
        let query_str = query.to_string();
        let result: Vec<Paper> = self
            .db
            .query(
                r#"
                SELECT * FROM paper
                WHERE deleted_at IS NONE
                AND (title @@ $query OR `abstract` @@ $query)
                ORDER BY id DESC
                LIMIT 50
                "#,
            )
            .bind(("query", query_str))
            .await
            .map_err(|e| AppError::generic(format!("Failed to search papers: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get search results: {}", e)))?;

        info!("Search for '{}' found {} papers", query, result.len());
        Ok(result)
    }

    /// Find papers by category
    pub async fn find_by_category(&self, category_id: &str) -> Result<Vec<Paper>> {
        let category_id = category_id.to_string();
        let result: Vec<Paper> = self
            .db
            .query(
                r#"
                SELECT * FROM paper
                WHERE deleted_at IS NONE
                AND id IN (SELECT `in` FROM paper_category WHERE `out` = type::thing($category_id))
                ORDER BY created_at DESC
                "#,
            )
            .bind(("category_id", category_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query papers by category: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }
}
