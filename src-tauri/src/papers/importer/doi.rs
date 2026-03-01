use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// DOI metadata fetcher error types
#[derive(Error, Debug)]
pub enum DoiError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid DOI format: {0}")]
    InvalidDoi(String),

    #[error("Failed to parse DOI metadata: {0}")]
    ParseError(String),

    #[error("DOI not found")]
    NotFound,
}

/// Metadata extracted from a DOI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoiMetadata {
    pub doi: String,
    pub title: String,
    pub authors: Vec<String>,
    pub publication_year: Option<String>,
    pub journal_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub publisher: Option<String>,
    pub url: Option<String>,
    pub abstract_text: Option<String>,
}

/// Helper type to deserialize title from either string or array of strings
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TitleField {
    String(String),
    Array(Vec<String>),
}

impl TitleField {
    fn into_string(self) -> Option<String> {
        match self {
            TitleField::String(s) => Some(s),
            TitleField::Array(arr) => arr.into_iter().next(),
        }
    }
}

/// Helper type to deserialize container title from either string or array of strings
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ContainerTitleField {
    String(String),
    Array(Vec<String>),
}

impl ContainerTitleField {
    fn into_string(self) -> Option<String> {
        match self {
            ContainerTitleField::String(s) => Some(s),
            ContainerTitleField::Array(arr) => arr.into_iter().next(),
        }
    }
}

/// Crossref metadata response structure
#[derive(Debug, Deserialize)]
struct CrossrefWork {
    #[serde(rename = "DOI")]
    doi: String,
    #[serde(rename = "type")]
    work_type: String,
    title: Option<TitleField>,
    #[serde(default)]
    author: Vec<CrossrefAuthor>,
    #[serde(rename = "published")]
    published: Option<serde_json::Value>,
    #[serde(rename = "short-container-title")]
    short_container_title: Option<ContainerTitleField>,
    #[serde(rename = "container-title")]
    container_title: Option<ContainerTitleField>,
    volume: Option<String>,
    issue: Option<String>,
    page: Option<String>,
    publisher: Option<String>,
    #[serde(rename = "URL")]
    url: Option<String>,
    abstract_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefAuthor {
    #[serde(rename = "given")]
    given_name: Option<String>,
    #[serde(rename = "family")]
    family_name: Option<String>,
    name: Option<String>,
}

impl CrossrefWork {
    /// Convert Crossref response to DoiMetadata
    #[allow(clippy::wrong_self_convention)]
    fn to_metadata(self) -> Result<DoiMetadata, DoiError> {
        let title = self
            .title
            .and_then(|t| t.into_string())
            .ok_or_else(|| DoiError::ParseError("Title not found".to_string()))?;

        let authors = self
            .author
            .into_iter()
            .map(|a| {
                if let Some(name) = a.name {
                    return name;
                }
                match (a.given_name, a.family_name) {
                    (Some(given), Some(family)) => format!("{} {}", given, family),
                    (Some(given), None) => given,
                    (None, Some(family)) => family,
                    (None, None) => "Unknown".to_string(),
                }
            })
            .collect();

        // Extract publication year from published date
        let publication_year = self.published.and_then(|p| {
            p.get("date-parts")
                .and_then(|parts| parts.as_array())
                .and_then(|arr| arr.first())
                .and_then(|part| part.as_array())
                .and_then(|arr| arr.first())
                .and_then(|year| year.as_i64())
                .map(|y| y.to_string())
        });

        let journal_name = self
            .short_container_title
            .or(self.container_title)
            .and_then(|t| t.into_string());

        Ok(DoiMetadata {
            doi: self.doi,
            title,
            authors,
            publication_year,
            journal_name,
            volume: self.volume,
            issue: self.issue,
            pages: self.page,
            publisher: self.publisher,
            url: self.url,
            abstract_text: self.abstract_text,
        })
    }
}

/// Fetch metadata for a given DOI
pub async fn fetch_doi_metadata(doi: &str) -> Result<DoiMetadata, DoiError> {
    // Validate DOI format
    if !is_valid_doi(doi) {
        return Err(DoiError::InvalidDoi(doi.to_string()));
    }

    // Build the DOI URL
    let url = format!("https://doi.org/{}", doi);

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("XuanBrain/0.1.0 (mailto:support@example.com)")
        .build()?;

    // Send request to DOI.org
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .await?;

    // Check response status
    let response = response.error_for_status().map_err(|e| {
        if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            DoiError::NotFound
        } else {
            DoiError::RequestError(e)
        }
    })?;

    // Parse response
    let crossref_work: CrossrefWork = response.json().await?;

    // Convert to metadata
    crossref_work.to_metadata()
}

/// Validate DOI format (basic check)
fn is_valid_doi(doi: &str) -> bool {
    // Basic DOI format validation: 10.xxx/xxx
    if doi.is_empty() {
        return false;
    }

    // Remove "doi:" prefix if present
    let doi = doi.strip_prefix("doi:").unwrap_or(doi);

    // Remove "https://doi.org/" prefix if present
    let doi = doi.strip_prefix("https://doi.org/").unwrap_or(doi);

    // Check basic format: starts with "10." followed by at least one digit, then "/", then at least one character
    let pattern = regex::Regex::new(r"^10\.\d+/.+$").unwrap();
    pattern.is_match(doi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_doi_metadata() {
        let doi = "10.1016/j.precisioneng.2019.10.013";

        let result = fetch_doi_metadata(doi).await;

        assert!(result.is_ok(), "Failed to fetch DOI metadata: {:?}", result);

        let metadata = result.unwrap();
        assert_eq!(metadata.doi, doi);
        assert!(!metadata.title.is_empty(), "Title should not be empty");
        assert!(!metadata.authors.is_empty(), "Authors should not be empty");

        println!("DOI: {}", metadata.doi);
        println!("Title: {}", metadata.title);
        println!("Authors: {}", metadata.authors.join(", "));
        println!("Year: {:?}", metadata.publication_year);
        println!("Journal: {:?}", metadata.journal_name);
        println!("Publisher: {:?}", metadata.publisher);
        println!("URL: {:?}", metadata.url);
    }

    /// Test to print detailed DOI metadata for inspection
    #[tokio::test]
    async fn test_print_doi_metadata_detail() {
        // Use a well-known DOI
        let doi = "10.1016/j.precisioneng.2019.10.013";

        println!("\n========== DOI Metadata Test ==========");
        println!("Fetching DOI: {}", doi);

        let result = fetch_doi_metadata(doi).await;

        match result {
            Ok(metadata) => {
                println!("\n--- Successfully fetched metadata ---");
                println!("DOI: {}", metadata.doi);
                println!("Title: {}", metadata.title);
                println!("Authors ({}):", metadata.authors.len());
                for (i, author) in metadata.authors.iter().enumerate() {
                    println!("  {}. {}", i + 1, author);
                }
                println!("Publication Year: {:?}", metadata.publication_year);
                println!("Journal Name: {:?}", metadata.journal_name);
                println!("Volume: {:?}", metadata.volume);
                println!("Issue: {:?}", metadata.issue);
                println!("Pages: {:?}", metadata.pages);
                println!("Publisher: {:?}", metadata.publisher);
                println!("URL: {:?}", metadata.url);
                println!(
                    "Abstract: {:?}",
                    metadata.abstract_text.as_ref().map(|s| {
                        if s.len() > 200 {
                            format!("{}...", &s[..200])
                        } else {
                            s.clone()
                        }
                    })
                );
            }
            Err(e) => {
                println!("Error fetching DOI: {}", e);
                panic!("Test failed: {:?}", e);
            }
        }
        println!("========== End DOI Test ==========\n");
    }

    #[test]
    fn test_is_valid_doi() {
        // Valid DOIs
        assert!(is_valid_doi("10.1016/j.precisioneng.2019.10.013"));
        assert!(is_valid_doi("10.1038/nature12373"));
        assert!(is_valid_doi("10.1109/5.771073"));
        assert!(is_valid_doi("doi:10.1016/j.precisioneng.2019.10.013"));
        assert!(is_valid_doi(
            "https://doi.org/10.1016/j.precisioneng.2019.10.013"
        ));

        // Invalid DOIs
        assert!(!is_valid_doi(""));
        assert!(!is_valid_doi("not-a-doi"));
        assert!(!is_valid_doi("11.1016/j.precisioneng.2019.10.013")); // Wrong prefix
        assert!(!is_valid_doi("10.1016")); // Missing suffix
        assert!(!is_valid_doi("10./test")); // Missing number
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_doi() {
        let result = fetch_doi_metadata("10.1234/nonexistent.doi.12345").await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(DoiError::NotFound) | Err(DoiError::RequestError(_))
        ));
    }

    #[tokio::test]
    async fn test_fetch_invalid_doi() {
        let result = fetch_doi_metadata("invalid-doi").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(DoiError::InvalidDoi(_))));
    }
}
