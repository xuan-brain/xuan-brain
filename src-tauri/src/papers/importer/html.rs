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
    #[serde(default, deserialize_with = "deserialize_null_vec")]
    pub authors: Vec<String>,
    pub doi: Option<String>,
    #[serde(rename = "abstract_text")]
    pub abstract_text: Option<String>,
    #[serde(rename = "journal")]
    pub journal_name: Option<String>,
    #[serde(rename = "year")]
    pub publication_year: Option<i64>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub source_domain: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_vec")]
    pub keywords: Vec<String>,
    pub extra: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Custom deserializer to handle null values for Vec fields
fn deserialize_null_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<Vec<String>>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
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

    // Call LLM API - system prompt is empty since the full prompt is in user_content
    let response = client.chat(provider, "", &user_content).await?;

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
            "abstract_text": "This is a test abstract.",
            "year": 2024,
            "journal": "Test Journal",
            "doi": "10.1000/test123",
            "volume": "10",
            "issue": "2",
            "pages": "1-15",
            "url": "https://example.com/paper",
            "source_domain": "example.com",
            "keywords": ["machine learning", "deep learning"],
            "extra": {"pii": "S12345678901234567"}
        }"#;

        let metadata: ExtractedPaperMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.title, "Test Paper Title");
        assert_eq!(metadata.authors.len(), 2);
        assert_eq!(metadata.publication_year, Some(2024));
        assert_eq!(metadata.journal_name, Some("Test Journal".to_string()));
        assert_eq!(metadata.keywords.len(), 2);
        assert!(metadata.extra.is_some());
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
        assert!(metadata.extra.is_none());
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

    #[test]
    fn test_parse_null_keywords_and_authors() {
        let json = r#"{
            "title": "Test Paper",
            "authors": null,
            "keywords": null
        }"#;

        let metadata: ExtractedPaperMetadata = serde_json::from_str(json).unwrap();
        assert_eq!(metadata.title, "Test Paper");
        assert!(metadata.authors.is_empty());
        assert!(metadata.keywords.is_empty());
    }
}
