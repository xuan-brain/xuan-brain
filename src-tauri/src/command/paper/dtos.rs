//! Data Transfer Objects for paper commands

use serde::{Deserialize, Serialize};

/// Batch DTO for streaming papers via Channel - uses lightweight PaperListDto
#[derive(Clone, Serialize)]
pub struct PaperBatchDto {
    /// Papers in this batch (lightweight, no attachments)
    pub papers: Vec<PaperListDto>,
    /// Index of this batch (0-based)
    pub batch_index: usize,
    /// Whether this is the last batch
    pub is_last: bool,
    /// Total number of papers loaded so far
    pub loaded_count: usize,
    /// Total number of papers in the database
    pub total: usize,
}

/// Initial response for streaming papers - contains first batch synchronously
#[derive(Clone, Serialize)]
pub struct StreamInitDto {
    /// First batch of papers (returned synchronously, lightweight)
    pub first_batch: Vec<PaperListDto>,
    /// Total number of papers in the database
    pub total: usize,
    /// Number of papers in first batch
    pub first_batch_count: usize,
    /// Whether there are more batches to load
    pub has_more: bool,
}

#[derive(Clone, Serialize)]
pub struct LabelDto {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Clone, Serialize)]
pub struct AttachmentDto {
    pub id: String,
    pub paper_id: String,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub created_at: Option<String>,
}

/// Result DTO for paper import operations
#[derive(Serialize)]
pub struct ImportResultDto {
    /// Whether the paper already exists in the database
    pub already_exists: bool,
    /// Message describing the result
    pub message: String,
    /// The paper data (None if already exists)
    pub paper: Option<PaperDto>,
}

#[derive(Serialize)]
pub struct PdfAttachmentInfo {
    pub file_path: String,
    pub file_name: String,
    pub paper_id: String,
    pub paper_title: String,
    pub base64_content: Option<String>,
}

#[derive(Serialize)]
pub struct PdfBlobResponse {
    pub file_name: String,
    pub paper_title: String,
    pub paper_id: String,
    pub base64_data: String,
    pub size_bytes: usize,
}

#[derive(Serialize)]
pub struct PdfSaveResponse {
    pub success: bool,
    pub file_path: String,
    pub size_bytes: usize,
    pub message: String,
}

#[derive(Clone, Serialize)]
pub struct PaperDto {
    pub id: String,
    pub title: String,
    pub publication_year: Option<i32>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub authors: Vec<String>,
    pub labels: Vec<LabelDto>,
    pub attachment_count: usize,
    pub attachments: Vec<AttachmentDto>,
    // New fields for Zotero import support
    pub publisher: Option<String>,
    pub issn: Option<String>,
    pub language: Option<String>,
}

/// Lightweight DTO for paper list view - optimized for fast serialization
/// Excludes heavy nested objects like attachments (only count is needed)
#[derive(Clone, Serialize)]
pub struct PaperListDto {
    pub id: String,
    pub title: String,
    pub publication_year: Option<i32>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub authors: Vec<String>,
    pub attachment_count: usize,
    // NOTE: attachments intentionally excluded - load on demand
    // NOTE: labels excluded - not displayed in table view
}

#[derive(Serialize)]
pub struct PaperDetailDto {
    pub id: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub publication_date: Option<String>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub citation_count: Option<i32>,
    pub read_status: Option<String>,
    pub notes: Option<String>,
    pub authors: Vec<String>,
    pub labels: Vec<LabelDto>,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub attachments: Vec<AttachmentDto>,
    pub attachment_count: usize,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    // New fields for Zotero import support
    pub publisher: Option<String>,
    pub issn: Option<String>,
    pub language: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePaperDto {
    pub id: String,
    pub title: String,
    pub publication_year: Option<i32>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
    pub read_status: Option<String>,
    // New fields for Zotero import support
    pub publisher: Option<String>,
    pub issn: Option<String>,
    pub language: Option<String>,
}

/// Result DTO for batch import operations (e.g., Zotero RDF import)
#[derive(Serialize)]
pub struct BatchImportResultDto {
    /// Total number of items processed
    pub total: usize,
    /// Number of items successfully imported
    pub imported: usize,
    /// Number of items skipped (duplicates)
    pub skipped: usize,
    /// Number of items that failed to import
    pub failed: usize,
    /// List of successfully imported papers
    pub papers: Vec<PaperDto>,
    /// List of error messages
    pub errors: Vec<String>,
}
