//! Data Transfer Objects for paper commands

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LabelDto {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct AttachmentDto {
    pub id: String,
    pub paper_id: String,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub created_at: Option<String>,
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

#[derive(Serialize)]
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
}
