//! Paper repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{attachment, paper, paper_category};
use crate::models::{Attachment, CreatePaper, Paper, UpdatePaper};
use crate::sys::error::{AppError, Result};

/// Repository for Paper operations
pub struct PaperRepository;

impl PaperRepository {
    /// Find all non-deleted papers
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Paper>> {
        let papers = paper::Entity::find()
            .filter(paper::Column::DeletedAt.is_null())
            .order_by_desc(paper::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query papers: {}", e)))?;

        info!("Found {} papers", papers.len());
        Ok(papers.into_iter().map(Paper::from).collect())
    }

    /// Find all deleted papers (trash)
    pub async fn find_deleted(db: &DatabaseConnection) -> Result<Vec<Paper>> {
        let papers = paper::Entity::find()
            .filter(paper::Column::DeletedAt.is_not_null())
            .order_by_desc(paper::Column::DeletedAt)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query deleted papers: {}", e)))?;

        Ok(papers.into_iter().map(Paper::from).collect())
    }

    /// Find paper by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Paper>> {
        let paper = paper::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper: {}", e)))?;

        Ok(paper.map(Paper::from))
    }

    /// Find paper by DOI
    pub async fn find_by_doi(db: &DatabaseConnection, doi: &str) -> Result<Option<Paper>> {
        let paper = paper::Entity::find()
            .filter(paper::Column::Doi.eq(doi))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query paper by DOI: {}", e)))?;

        Ok(paper.map(Paper::from))
    }

    /// Find paper by URL
    pub async fn find_by_url(db: &DatabaseConnection, url: &str) -> Result<Option<Paper>> {
        let paper = paper::Entity::find()
            .filter(paper::Column::Url.eq(url))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query paper by URL: {}", e)))?;

        Ok(paper.map(Paper::from))
    }

    /// Create a new paper
    pub async fn create(db: &DatabaseConnection, create: CreatePaper) -> Result<Paper> {
        let now = chrono::Utc::now();
        let new_paper = paper::ActiveModel {
            title: Set(create.title),
            abstract_text: Set(create.abstract_text),
            doi: Set(create.doi),
            publication_year: Set(create.publication_year),
            publication_date: Set(create.publication_date),
            journal_name: Set(create.journal_name),
            conference_name: Set(create.conference_name),
            volume: Set(create.volume),
            issue: Set(create.issue),
            pages: Set(create.pages),
            url: Set(create.url),
            citation_count: Set(0),
            read_status: Set("unread".to_string()),
            notes: Set(None),
            attachment_path: Set(create.attachment_path),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
            ..Default::default()
        };

        let result = new_paper
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create paper: {}", e)))?;

        Ok(Paper::from(result))
    }

    /// Update paper
    pub async fn update(db: &DatabaseConnection, id: i64, update: UpdatePaper) -> Result<Paper> {
        let paper = paper::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find paper: {}", e)))?
            .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

        let mut paper: paper::ActiveModel = paper.into();
        if let Some(title) = update.title {
            paper.title = Set(title);
        }
        if let Some(abstract_text) = update.abstract_text {
            paper.abstract_text = Set(Some(abstract_text));
        }
        if let Some(doi) = update.doi {
            paper.doi = Set(Some(doi));
        }
        if let Some(publication_year) = update.publication_year {
            paper.publication_year = Set(Some(publication_year));
        }
        if let Some(publication_date) = update.publication_date {
            paper.publication_date = Set(Some(publication_date));
        }
        if let Some(journal_name) = update.journal_name {
            paper.journal_name = Set(Some(journal_name));
        }
        if let Some(conference_name) = update.conference_name {
            paper.conference_name = Set(Some(conference_name));
        }
        if let Some(volume) = update.volume {
            paper.volume = Set(Some(volume));
        }
        if let Some(issue) = update.issue {
            paper.issue = Set(Some(issue));
        }
        if let Some(pages) = update.pages {
            paper.pages = Set(Some(pages));
        }
        if let Some(url) = update.url {
            paper.url = Set(Some(url));
        }
        if let Some(read_status) = update.read_status {
            paper.read_status = Set(read_status);
        }
        if let Some(notes) = update.notes {
            paper.notes = Set(Some(notes));
        }
        if let Some(attachment_path) = update.attachment_path {
            paper.attachment_path = Set(Some(attachment_path));
        }

        paper.updated_at = Set(chrono::Utc::now());

        let result = paper
            .update(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to update paper: {}", e)))?;

        Ok(Paper::from(result))
    }

    /// Soft delete paper (move to trash)
    pub async fn soft_delete(db: &DatabaseConnection, id: i64) -> Result<()> {
        let paper = paper::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find paper: {}", e)))?
            .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

        let mut paper: paper::ActiveModel = paper.into();
        paper.deleted_at = Set(Some(chrono::Utc::now()));
        paper.update(db).await.map_err(|e| {
            AppError::generic(format!("Failed to soft delete paper: {}", e))
        })?;

        Ok(())
    }

    /// Restore paper from trash
    pub async fn restore(db: &DatabaseConnection, id: i64) -> Result<()> {
        let paper = paper::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find paper: {}", e)))?
            .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

        let mut paper: paper::ActiveModel = paper.into();
        paper.deleted_at = Set(None);
        paper.update(db).await.map_err(|e| {
            AppError::generic(format!("Failed to restore paper: {}", e))
        })?;

        Ok(())
    }

    /// Permanently delete paper
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
        paper::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete paper: {}", e)))?;

        Ok(())
    }

    /// Search papers using LIKE query (basic search)
    pub async fn search(db: &DatabaseConnection, query: &str) -> Result<Vec<Paper>> {
        let pattern = format!("%{}%", query);
        let papers = paper::Entity::find()
            .filter(paper::Column::DeletedAt.is_null())
            .filter(
                Condition::any()
                    .add(paper::Column::Title.like(&pattern))
                    .add(paper::Column::AbstractText.like(&pattern)),
            )
            .order_by_desc(paper::Column::Id)
            .limit(50)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to search papers: {}", e)))?;

        info!("Search for '{}' found {} papers", query, papers.len());
        Ok(papers.into_iter().map(Paper::from).collect())
    }

    /// Find papers by category
    pub async fn find_by_category(db: &DatabaseConnection, category_id: i64) -> Result<Vec<Paper>> {
        // First get paper_category relations
        let relations = paper_category::Entity::find()
            .filter(paper_category::Column::CategoryId.eq(category_id))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper-category relations: {}", e)))?;

        let paper_ids: Vec<i64> = relations.iter().map(|r| r.paper_id).collect();

        if paper_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Then get papers by IDs
        let papers = paper::Entity::find()
            .filter(paper::Column::Id.is_in(paper_ids))
            .filter(paper::Column::DeletedAt.is_null())
            .order_by_desc(paper::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query papers by category: {}", e)))?;

        Ok(papers.into_iter().map(Paper::from).collect())
    }

    /// Set paper category (replaces existing category)
    pub async fn set_category(
        db: &DatabaseConnection,
        paper_id: i64,
        category_id: Option<i64>,
    ) -> Result<()> {
        // First delete existing category relation
        paper_category::Entity::delete_many()
            .filter(paper_category::Column::PaperId.eq(paper_id))
            .exec(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to delete paper category: {}", e))
            })?;

        // Then create new relation if category_id is provided
        if let Some(cat_id) = category_id {
            let relation = paper_category::ActiveModel {
                paper_id: Set(paper_id),
                category_id: Set(cat_id),
                ..Default::default()
            };
            relation.insert(db).await.map_err(|e| {
                AppError::generic(format!("Failed to set paper category: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get paper's category ID
    pub async fn get_category_id(db: &DatabaseConnection, paper_id: i64) -> Result<Option<i64>> {
        let relation = paper_category::Entity::find()
            .filter(paper_category::Column::PaperId.eq(paper_id))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper category: {}", e)))?;

        Ok(relation.map(|r| r.category_id))
    }

    /// Update attachment path
    pub async fn update_attachment_path(
        db: &DatabaseConnection,
        paper_id: i64,
        path: &str,
    ) -> Result<()> {
        let paper = paper::Entity::find_by_id(paper_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find paper: {}", e)))?
            .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

        let mut paper: paper::ActiveModel = paper.into();
        paper.attachment_path = Set(Some(path.to_string()));
        paper.updated_at = Set(chrono::Utc::now());
        paper.update(db).await.map_err(|e| {
            AppError::generic(format!("Failed to update attachment path: {}", e))
        })?;

        Ok(())
    }

    // ==================== Attachment operations ====================

    /// Add attachment to paper
    pub async fn add_attachment(
        db: &DatabaseConnection,
        paper_id: i64,
        file_name: Option<String>,
        file_type: Option<String>,
        file_size: Option<i64>,
    ) -> Result<Attachment> {
        let now = chrono::Utc::now();
        let new_attachment = attachment::ActiveModel {
            paper_id: Set(paper_id),
            file_name: Set(file_name),
            file_type: Set(file_type),
            file_size: Set(file_size),
            created_at: Set(now),
            ..Default::default()
        };

        let result = new_attachment
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add attachment: {}", e)))?;

        // Update paper's updated_at
        Self::touch_paper(db, paper_id).await?;

        Ok(Attachment::from(result))
    }

    /// Get all attachments for a paper
    pub async fn get_attachments(db: &DatabaseConnection, paper_id: i64) -> Result<Vec<Attachment>> {
        let attachments = attachment::Entity::find()
            .filter(attachment::Column::PaperId.eq(paper_id))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get attachments: {}", e)))?;

        Ok(attachments.into_iter().map(Attachment::from).collect())
    }

    /// Find PDF attachment for a paper
    pub async fn find_pdf_attachment(
        db: &DatabaseConnection,
        paper_id: i64,
    ) -> Result<Option<Attachment>> {
        let attachments = Self::get_attachments(db, paper_id).await?;
        Ok(attachments.into_iter().find(|a| {
            let file_type = a.file_type.as_deref().unwrap_or("").to_lowercase();
            let file_name = a.file_name.as_deref().unwrap_or("");
            file_type == "pdf" || file_name.ends_with(".pdf")
        }))
    }

    /// Remove attachment from paper by ID
    pub async fn remove_attachment(db: &DatabaseConnection, attachment_id: i64) -> Result<()> {
        // Get attachment to find paper_id
        let attachment = attachment::Entity::find_by_id(attachment_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find attachment: {}", e)))?
            .ok_or_else(|| AppError::not_found("Attachment", attachment_id.to_string()))?;

        let paper_id = attachment.paper_id;

        // Delete attachment
        attachment::Entity::delete_by_id(attachment_id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to remove attachment: {}", e)))?;

        // Update paper's updated_at
        Self::touch_paper(db, paper_id).await?;

        Ok(())
    }

    /// Remove attachment from paper by file name
    pub async fn remove_attachment_by_name(
        db: &DatabaseConnection,
        paper_id: i64,
        file_name: &str,
    ) -> Result<()> {
        attachment::Entity::delete_many()
            .filter(attachment::Column::PaperId.eq(paper_id))
            .filter(attachment::Column::FileName.eq(file_name))
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to remove attachment: {}", e)))?;

        // Update paper's updated_at
        Self::touch_paper(db, paper_id).await?;

        Ok(())
    }

    /// Update paper's updated_at timestamp
    async fn touch_paper(db: &DatabaseConnection, paper_id: i64) -> Result<()> {
        let paper = paper::Entity::find_by_id(paper_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find paper: {}", e)))?;

        if let Some(paper) = paper {
            let mut paper: paper::ActiveModel = paper.into();
            paper.updated_at = Set(chrono::Utc::now());
            paper.update(db).await.map_err(|e| {
                AppError::generic(format!("Failed to update paper timestamp: {}", e))
            })?;
        }

        Ok(())
    }

    // ==================== Author operations ====================

    /// Add author to paper
    pub async fn add_author(
        db: &DatabaseConnection,
        paper_id: i64,
        author_id: i64,
        author_order: i32,
    ) -> Result<()> {
        use crate::database::entities::paper_author;

        let relation = paper_author::ActiveModel {
            paper_id: Set(paper_id),
            author_id: Set(author_id),
            author_order: Set(author_order),
            is_corresponding: Set(0),
            ..Default::default()
        };

        relation
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add author: {}", e)))?;

        Ok(())
    }

    /// Add attachment from model
    pub async fn add_attachment_model(
        db: &DatabaseConnection,
        attachment: crate::models::Attachment,
    ) -> Result<Attachment> {
        let new_attachment = attachment::ActiveModel {
            paper_id: Set(attachment.paper_id),
            file_name: Set(attachment.file_name),
            file_type: Set(attachment.file_type),
            file_size: Set(attachment.file_size),
            created_at: Set(attachment.created_at),
            ..Default::default()
        };

        let result = new_attachment
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add attachment: {}", e)))?;

        Ok(Attachment::from(result))
    }
}
