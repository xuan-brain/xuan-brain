//! Attachment repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{Attachment, CreateAttachment};
use crate::sys::error::{AppError, Result};
use tracing::info;

/// Repository for Attachment operations
pub struct AttachmentRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> AttachmentRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Find all attachments for a paper
    pub async fn find_by_paper(&self, paper_id: &str) -> Result<Vec<Attachment>> {
        let paper_id = paper_id.to_string();
        let result: Vec<Attachment> = self
            .db
            .query(
                r#"
                SELECT * FROM attachment
                WHERE paper = type::record($paper)
                ORDER BY created_at DESC
                "#,
            )
            .bind(("paper", paper_id.clone()))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query attachments: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!(
            "Found {} attachments for paper for paper id :{}",
            result.len(),
            &paper_id
        );
        Ok(result)
    }

    /// Find PDF attachment for a paper
    pub async fn find_pdf_by_paper(&self, paper_id: &str) -> Result<Option<Attachment>> {
        let paper_id = paper_id.to_string();
        let result: Vec<Attachment> = self
            .db
            .query(
                r#"
                SELECT * FROM attachment
                WHERE paper = type::record($paper)
                AND (file_type = "pdf" OR file_type = "application/pdf" OR string::lowercase(file_name) CONTAINS ".pdf")
                LIMIT 1
                "#,
            )
            .bind(("paper", paper_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query PDF attachment: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Find attachment by ID
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Attachment>> {
        let id = id.to_string();
        let result: Vec<Attachment> = self
            .db
            .query("SELECT * FROM type::record($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get attachment: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new attachment
    pub async fn create(&self, attachment: CreateAttachment) -> Result<Attachment> {
        let paper_id = attachment.paper_id.clone();
        let file_type = attachment.file_type.clone();
        let file_name = attachment.file_name.clone();
        let file_path = attachment.file_path.clone();
        let file_size = attachment.file_size;

        let result: Vec<Attachment> = self
            .db
            .query(
                r#"
                CREATE attachment SET
                    paper = type::record($paper_id),
                    file_type = $file_type,
                    file_name = $file_name,
                    file_path = $file_path,
                    file_size = $file_size,
                    created_at = time::now()
                "#,
            )
            .bind(("paper_id", paper_id))
            .bind(("file_type", file_type))
            .bind(("file_name", file_name))
            .bind(("file_path", file_path))
            .bind(("file_size", file_size))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create attachment: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create attachment".to_string()))
    }

    /// Delete attachment by ID
    pub async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        self.db
            .query("DELETE type::record($id)")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete attachment: {}", e)))?;

        Ok(())
    }

    /// Delete all attachments for a paper
    pub async fn delete_by_paper(&self, paper_id: &str) -> Result<()> {
        let paper_id = paper_id.to_string();
        self.db
            .query("DELETE attachment WHERE paper = type::record($paper)")
            .bind(("paper", paper_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete attachments: {}", e)))?;

        Ok(())
    }
}
