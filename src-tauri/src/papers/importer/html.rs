use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;

use crate::llm::client::{LlmClient, LlmError};
use crate::llm::prompts::HTML_PAPER_EXTRACTION_PROMPT;
use crate::sys::config::LlmProvider;

/// HTML import error types
#[derive(Error, Debug)]
pub enum HtmlImportError {
    #[error("LLM error: {0}")]
    LlmError(#[from] LlmError),

    #[error("Failed to parse AI response: {0}")]
    ParseError(String),

    #[error("No valid metadata found in HTML")]
    NoMetadata,

    #[error("Title is required but not found")]
    MissingTitle,

    #[error("AI returned an error: {0}")]
    AiError(String),
}

/// Metadata extracted from HTML by AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedPaperMetadata {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    pub publication_year: Option<i64>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Extract paper metadata from HTML content using AI
pub async fn extract_paper_from_html(
    html_content: &str,
    provider: &LlmProvider,
) -> Result<ExtractedPaperMetadata, HtmlImportError> {
    info!("Extracting paper metadata from HTML using LLM");

    let client = LlmClient::new();

    // Build the prompt with HTML content
    let user_content = format!("{}{}", HTML_PAPER_EXTRACTION_PROMPT, html_content);

    // Call LLM API
    let response = client.chat(provider, "You are a scholarly paper metadata extraction assistant.", &user_content).await?;

    info!("LLM response received, parsing metadata");

    // Clean up the response - remove markdown code blocks if present
    let cleaned_response = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    // Parse JSON response
    let metadata: ExtractedPaperMetadata =
        serde_json::from_str(cleaned_response).map_err(|e| {
            HtmlImportError::ParseError(format!("Failed to parse JSON response: {}. Response: {}", e, cleaned_response))
        })?;

    // Check if AI returned an error
    if let Some(error) = &metadata.error {
        return Err(HtmlImportError::AiError(error.clone()));
    }

    // Validate required fields
    if metadata.title.is_empty() {
        return Err(HtmlImportError::MissingTitle);
    }

    info!("Successfully extracted metadata for paper: {}", metadata.title);

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_extracted_metadata() {
        let json = r#"{
            "title": "Test Paper Title",
            "authors": ["Author 1", "Author 2"],
            "abstract": "This is a test abstract.",
            "publication_year": 2024,
            "journal_name": "Test Journal",
            "doi": "10.1000/test123"
        }"#;

        let metadata: ExtractedPaperMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.title, "Test Paper Title");
        assert_eq!(metadata.authors.len(), 2);
        assert_eq!(metadata.publication_year, Some(2024));
    }

    #[test]
    fn test_parse_minimal_metadata() {
        let json = r#"{
            "title": "Minimal Paper"
        }"#;

        let metadata: ExtractedPaperMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.title, "Minimal Paper");
        assert!(metadata.authors.is_empty());
        assert!(metadata.publication_year.is_none());
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{
            "title": "",
            "error": "Title not found in HTML"
        }"#;

        let metadata: ExtractedPaperMetadata = serde_json::from_str(json).unwrap();
        assert!(metadata.error.is_some());
        assert_eq!(metadata.error.unwrap(), "Title not found in HTML");
    }
}
