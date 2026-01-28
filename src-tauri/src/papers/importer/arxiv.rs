use reqwest::header::ACCEPT;
use serde::Deserialize;
use thiserror::Error;

/// arXiv metadata fetcher error types
#[derive(Error, Debug)]
pub enum ArxivError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid arXiv ID format: {0}")]
    InvalidArxivId(String),

    #[error("Failed to parse arXiv metadata: {0}")]
    ParseError(String),

    #[error("arXiv paper not found")]
    NotFound,
}

/// Metadata extracted from an arXiv paper
#[derive(Debug, Clone, Deserialize)]
pub struct ArxivMetadata {
    pub arxiv_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub summary: String,
    pub published: String,
    pub updated: String,
    pub primary_category: String,
    pub categories: Vec<String>,
    pub pdf_url: String,
    pub doi: Option<String>,
    pub journal_ref: Option<String>,
}

/// arXiv Atom entry structure
#[derive(Debug, Deserialize)]
struct ArxivEntry {
    id: String,
    updated: String,
    published: String,
    title: String,
    summary: String,
    author: Vec<ArxivAuthor>,
    #[serde(rename = "primary_category")]
    primary_category: ArxivCategory,
    category: Vec<ArxivCategory>,
    link: Vec<ArxivLink>,
    #[serde(rename = "arxiv:doi", default)]
    doi: Option<String>,
    #[serde(rename = "arxiv:journal_ref", default)]
    journal_ref: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ArxivAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ArxivCategory {
    #[serde(rename = "@term")]
    term: String,
}

#[derive(Debug, Deserialize)]
struct ArxivLink {
    #[serde(rename = "@href")]
    href: String,
    #[serde(rename = "@type")]
    link_type: Option<String>,
    #[serde(rename = "@title", default)]
    title: Option<String>,
}

/// arXiv Atom feed response
#[derive(Debug, Deserialize)]
struct ArxivFeed {
    entry: Vec<ArxivEntry>,
}

impl ArxivEntry {
    /// Convert arXiv entry to metadata
    fn to_metadata(&self) -> Result<ArxivMetadata, ArxivError> {
        // Clean up title (remove whitespace and newlines)
        let title = self
            .title
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        if title.is_empty() {
            return Err(ArxivError::ParseError("Title is empty".to_string()));
        }

        let authors = self.author.iter().map(|a| a.name.clone()).collect();

        let categories = self.category.iter().map(|c| c.term.clone()).collect();

        // Find PDF URL
        let pdf_url = self
            .link
            .iter()
            .find(|l| l.title.as_deref() == Some("pdf"))
            .map(|l| l.href.clone())
            .ok_or_else(|| ArxivError::ParseError("PDF URL not found".to_string()))?;

        // Extract arXiv ID from the ID field
        let arxiv_id = extract_arxiv_id_from_url(&self.id)
            .ok_or_else(|| ArxivError::ParseError("Failed to extract arXiv ID".to_string()))?;

        Ok(ArxivMetadata {
            arxiv_id,
            title,
            authors,
            summary: self.summary.trim().to_string(),
            published: self.published.clone(),
            updated: self.updated.clone(),
            primary_category: self.primary_category.term.clone(),
            categories,
            pdf_url,
            doi: self.doi.clone(),
            journal_ref: self.journal_ref.clone(),
        })
    }
}

/// Extract arXiv ID from various formats
pub fn extract_arxiv_id(arxiv_input: &str) -> Option<String> {
    if arxiv_input.is_empty() {
        return None;
    }

    // Remove "arXiv:" prefix if present
    let input = arxiv_input.strip_prefix("arXiv:").unwrap_or(arxiv_input);

    // Remove "arxiv:" prefix (lowercase)
    let input = input.strip_prefix("arxiv:").unwrap_or(input);

    // Extract from URL
    if input.contains("arxiv.org/") {
        // Handle abs/ URL format
        if let Some(start) = input.find("abs/") {
            let start = start + 4;
            if let Some(end) = input[start..].find(&['/', '?', 'v'][..]) {
                return Some(input[start..start + end].to_string());
            } else {
                return Some(input[start..].to_string());
            }
        }
        // Handle pdf/ URL format
        if let Some(start) = input.find("pdf/") {
            let start = start + 4;
            let remaining = &input[start..];
            // Remove .pdf extension
            let id = remaining.strip_suffix(".pdf").unwrap_or(remaining);
            // Remove version suffix (e.g., v2)
            if let Some(pos) = id.rfind('v') {
                // Check if version is followed by digits
                if id.len() > pos + 1
                    && id[pos + 1..]
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                {
                    return Some(id[..pos].to_string());
                }
            }
            return Some(id.to_string());
        }
    }

    // Validate arXiv ID format: YYMM.NNNNN or arch-YY/NNNNNN
    let modern_pattern = regex::Regex::new(r"^\d{4}\.\d{4,5}$").unwrap();
    let old_pattern = regex::Regex::new(r"^[a-z-]+/\d{6,}$").unwrap();

    if modern_pattern.is_match(input) || old_pattern.is_match(input) {
        Some(input.to_string())
    } else {
        None
    }
}

/// Extract arXiv ID from URL
fn extract_arxiv_id_from_url(url: &str) -> Option<String> {
    if let Some(start) = url.find("abs/") {
        let start = start + 4;
        if let Some(end) = url[start..].find(&['/', '?', 'v'][..]) {
            return Some(url[start..start + end].to_string());
        } else {
            return Some(url[start..].to_string());
        }
    }
    None
}

/// Fetch metadata for a given arXiv ID
pub async fn fetch_arxiv_metadata(arxiv_id: &str) -> Result<ArxivMetadata, ArxivError> {
    // Extract and validate arXiv ID
    let extracted_id = extract_arxiv_id(arxiv_id)
        .ok_or_else(|| ArxivError::InvalidArxivId(arxiv_id.to_string()))?;

    // Build the arXiv API URL
    let url = format!(
        "https://export.arxiv.org/api/query?id_list={}",
        extracted_id
    );

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("XuanBrain/0.1.0 (mailto:support@example.com)")
        .build()?;

    // Send request to arXiv API
    let response = client
        .get(&url)
        .header(ACCEPT, "application/atom+xml")
        .send()
        .await?;

    // Check response status
    let response = response.error_for_status().map_err(|e| {
        if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            ArxivError::NotFound
        } else {
            ArxivError::RequestError(e)
        }
    })?;

    // Parse XML response
    let xml_text = response.text().await?;
    let feed: ArxivFeed = quick_xml::de::from_str(&xml_text)
        .map_err(|e| ArxivError::ParseError(format!("XML parse error: {}", e)))?;

    // Get the first (and should be only) entry
    let entry = feed
        .entry
        .into_iter()
        .next()
        .ok_or_else(|| ArxivError::NotFound)?;

    entry.to_metadata()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_arxiv_id() {
        // Modern format: YYMM.NNNNN
        assert_eq!(
            extract_arxiv_id("2301.12345"),
            Some("2301.12345".to_string())
        );

        // With prefix
        assert_eq!(
            extract_arxiv_id("arXiv:2301.12345"),
            Some("2301.12345".to_string())
        );

        // From URL (abs)
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/abs/2301.12345"),
            Some("2301.12345".to_string())
        );

        // From URL (pdf)
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/pdf/2301.12345.pdf"),
            Some("2301.12345".to_string())
        );

        // Old format
        assert_eq!(
            extract_arxiv_id("math-ph/0503007"),
            Some("math-ph/0503007".to_string())
        );

        // With version number
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/abs/2301.12345v2"),
            Some("2301.12345".to_string())
        );

        // Invalid formats
        assert_eq!(extract_arxiv_id(""), None);
        assert_eq!(extract_arxiv_id("not-an-arxiv-id"), None);
        assert_eq!(extract_arxiv_id("1234"), None);
    }

    #[tokio::test]
    async fn test_fetch_arxiv_metadata() {
        let arxiv_id = "2301.12345"; // A known arXiv paper

        let result = fetch_arxiv_metadata(arxiv_id).await;

        assert!(
            result.is_ok(),
            "Failed to fetch arXiv metadata: {:?}",
            result
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.arxiv_id, "2301.12345");
        assert!(!metadata.title.is_empty(), "Title should not be empty");
        assert!(!metadata.authors.is_empty(), "Authors should not be empty");
        assert!(!metadata.summary.is_empty(), "Summary should not be empty");
        assert!(metadata.pdf_url.starts_with("https://arxiv.org/pdf/"));

        println!("arXiv ID: {}", metadata.arxiv_id);
        println!("Title: {}", metadata.title);
        println!("Authors: {}", metadata.authors.join(", "));
        println!("Published: {}", metadata.published);
        println!("Primary Category: {}", metadata.primary_category);
        println!(
            "Summary: {}...",
            metadata.summary.chars().take(100).collect::<String>()
        );
    }

    #[tokio::test]
    async fn test_fetch_invalid_arxiv_id() {
        let result = fetch_arxiv_metadata("9999.99999").await;
        // This might return NotFound or ParseError depending on API response
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_arxiv_id() {
        let result = fetch_arxiv_metadata("invalid-format").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ArxivError::InvalidArxivId(_))));
    }
}
