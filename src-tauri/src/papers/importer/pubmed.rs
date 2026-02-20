use reqwest::header::ACCEPT;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// PubMed metadata fetcher error types
#[derive(Error, Debug)]
pub enum PubmedError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid PMID format: {0}")]
    InvalidPmid(String),

    #[error("Failed to parse PubMed metadata: {0}")]
    ParseError(String),

    #[error("PubMed article not found")]
    NotFound,

    #[error("XML parsing error: {0}")]
    XmlError(String),
}

/// Metadata extracted from a PubMed article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubmedMetadata {
    pub pmid: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: Option<String>,
    pub journal_name: Option<String>,
    pub publication_year: Option<String>,
    pub publication_month: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub doi: Option<String>,
    pub pmc_id: Option<String>,
    pub keywords: Vec<String>,
    pub mesh_terms: Vec<String>,
}

/// PubMed XML response structures
#[derive(Debug, Deserialize)]
struct PubmedArticleSet {
    #[serde(rename = "PubmedArticle", default)]
    pubmed_articles: Vec<PubmedArticle>,
}

#[derive(Debug, Deserialize)]
struct PubmedArticle {
    #[serde(rename = "MedlineCitation")]
    medline_citation: MedlineCitation,
    #[serde(rename = "PubmedData")]
    pubmed_data: Option<PubmedData>,
}

#[derive(Debug, Deserialize)]
struct MedlineCitation {
    #[serde(rename = "PMID")]
    pmid: PmidField,
    #[serde(rename = "Article")]
    article: Article,
    #[serde(rename = "MeshHeadingList", default)]
    mesh_heading_list: Option<MeshHeadingList>,
    #[serde(rename = "KeywordList", default)]
    keyword_list: Option<KeywordList>,
}

#[derive(Debug, Deserialize)]
struct PmidField {
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct Article {
    #[serde(rename = "ArticleTitle")]
    article_title: String,
    #[serde(rename = "Abstract", default)]
    abstract_text: Option<Abstract>,
    #[serde(rename = "AuthorList", default)]
    author_list: Option<AuthorList>,
    #[serde(rename = "Journal")]
    journal: Journal,
}

#[derive(Debug, Deserialize)]
struct Abstract {
    #[serde(rename = "AbstractText", default)]
    abstract_texts: Vec<AbstractText>,
}

#[derive(Debug, Deserialize)]
struct AbstractText {
    #[serde(rename = "@Label", default)]
    label: Option<String>,
    #[serde(rename = "$value", default)]
    value: Option<String>,
    // For structured abstracts with nested text
    #[serde(rename = "", default)]
    text: Option<String>,
}

impl Abstract {
    fn to_string(&self) -> Option<String> {
        if self.abstract_texts.is_empty() {
            return None;
        }

        let parts: Vec<String> = self
            .abstract_texts
            .iter()
            .filter_map(|at| {
                let text = at.value.as_ref().or(at.text.as_ref())?;
                if let Some(label) = &at.label {
                    Some(format!("{}: {}", label, text))
                } else {
                    Some(text.clone())
                }
            })
            .collect();

        if parts.is_empty() {
            None
        } else {
            Some(parts.join("\n\n"))
        }
    }
}

#[derive(Debug, Deserialize)]
struct AuthorList {
    #[serde(rename = "Author", default)]
    authors: Vec<Author>,
}

#[derive(Debug, Deserialize)]
struct Author {
    #[serde(rename = "ForeName", default)]
    fore_name: Option<String>,
    #[serde(rename = "LastName", default)]
    last_name: Option<String>,
    #[serde(rename = "CollectiveName", default)]
    collective_name: Option<String>,
}

impl Author {
    fn to_string(&self) -> Option<String> {
        if let Some(collective) = &self.collective_name {
            return Some(collective.clone());
        }

        match (&self.fore_name, &self.last_name) {
            (Some(fore), Some(last)) => Some(format!("{} {}", fore, last)),
            (Some(fore), None) => Some(fore.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Journal {
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "ISOAbbreviation")]
    iso_abbreviation: Option<String>,
    #[serde(rename = "JournalIssue")]
    journal_issue: Option<JournalIssue>,
}

#[derive(Debug, Deserialize)]
struct JournalIssue {
    #[serde(rename = "Volume")]
    volume: Option<String>,
    #[serde(rename = "Issue")]
    issue: Option<String>,
    #[serde(rename = "PubDate")]
    pub_date: Option<PubDate>,
}

#[derive(Debug, Deserialize)]
struct PubDate {
    #[serde(rename = "Year")]
    year: Option<String>,
    #[serde(rename = "Month")]
    month: Option<String>,
    #[serde(rename = "MedlineDate")]
    medline_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PubmedData {
    #[serde(rename = "ArticleIdList")]
    article_id_list: Option<ArticleIdList>,
}

#[derive(Debug, Deserialize)]
struct ArticleIdList {
    #[serde(rename = "ArticleId", default)]
    article_ids: Vec<ArticleId>,
}

#[derive(Debug, Deserialize)]
struct ArticleId {
    #[serde(rename = "@IdType")]
    id_type: String,
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct MeshHeadingList {
    #[serde(rename = "MeshHeading", default)]
    mesh_headings: Vec<MeshHeading>,
}

#[derive(Debug, Deserialize)]
struct MeshHeading {
    #[serde(rename = "DescriptorName")]
    descriptor_name: DescriptorName,
}

#[derive(Debug, Deserialize)]
struct DescriptorName {
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Deserialize)]
struct KeywordList {
    #[serde(rename = "Keyword", default)]
    keywords: Vec<Keyword>,
}

#[derive(Debug, Deserialize)]
struct Keyword {
    #[serde(rename = "$value")]
    value: String,
}

impl PubmedArticle {
    /// Convert PubMed article to metadata
    fn to_metadata(&self) -> Result<PubmedMetadata, PubmedError> {
        // Clean up title (remove whitespace and newlines)
        let title = self
            .medline_citation
            .article
            .article_title
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ");

        if title.is_empty() {
            return Err(PubmedError::ParseError("Title is empty".to_string()));
        }

        // Extract authors
        let authors = self
            .medline_citation
            .article
            .author_list
            .as_ref()
            .map(|al| al.authors.iter().filter_map(|a| a.to_string()).collect())
            .unwrap_or_default();

        // Extract abstract
        let abstract_text = self
            .medline_citation
            .article
            .abstract_text
            .as_ref()
            .and_then(|a| a.to_string());

        // Extract journal info
        let journal = &self.medline_citation.article.journal;
        let journal_name = journal
            .title
            .as_ref()
            .or(journal.iso_abbreviation.as_ref())
            .cloned();

        // Extract publication date, volume, issue
        let (publication_year, publication_month, volume, issue) = journal
            .journal_issue
            .as_ref()
            .map_or((None, None, None, None), |ji| {
                let (year, month) = ji.pub_date.as_ref().map_or((None, None), |pd| {
                    // Try to extract year from MedlineDate if Year is not available
                    let year = pd.year.clone().or_else(|| {
                        pd.medline_date.as_ref().and_then(|md| {
                            // Extract first 4 digits as year
                            md.chars()
                                .take(4)
                                .collect::<String>()
                                .parse::<u32>()
                                .ok()
                                .map(|y| y.to_string())
                        })
                    });
                    (year, pd.month.clone())
                });
                (year, month, ji.volume.clone(), ji.issue.clone())
            });

        // Extract pages from pagination if available
        let pages = None; // Pagination is complex, skip for now

        // Extract DOI and PMC ID
        let (doi, pmc_id) = self
            .pubmed_data
            .as_ref()
            .and_then(|pd| pd.article_id_list.as_ref())
            .map(|ail| {
                let doi = ail
                    .article_ids
                    .iter()
                    .find(|aid| aid.id_type == "doi")
                    .map(|aid| aid.value.clone());
                let pmc_id = ail
                    .article_ids
                    .iter()
                    .find(|aid| aid.id_type == "pmc")
                    .map(|aid| aid.value.clone());
                (doi, pmc_id)
            })
            .unwrap_or((None, None));

        // Extract MeSH terms
        let mesh_terms = self
            .medline_citation
            .mesh_heading_list
            .as_ref()
            .map(|mhl| {
                mhl.mesh_headings
                    .iter()
                    .map(|mh| mh.descriptor_name.value.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Extract keywords
        let keywords = self
            .medline_citation
            .keyword_list
            .as_ref()
            .map(|kl| kl.keywords.iter().map(|k| k.value.clone()).collect())
            .unwrap_or_default();

        Ok(PubmedMetadata {
            pmid: self.medline_citation.pmid.value.clone(),
            title,
            authors,
            abstract_text,
            journal_name,
            publication_year,
            publication_month,
            volume,
            issue,
            pages,
            doi,
            pmc_id,
            keywords,
            mesh_terms,
        })
    }
}

/// Validate PMID format (should be a numeric string)
fn is_valid_pmid(pmid: &str) -> bool {
    if pmid.is_empty() {
        return false;
    }

    // Remove "PMID:" prefix if present (case-insensitive)
    let pmid = pmid
        .strip_prefix("PMID:")
        .or_else(|| pmid.strip_prefix("pmid:"))
        .unwrap_or(pmid);

    // Remove URL prefix if present
    let pmid = if pmid.contains("pubmed/") {
        pmid.split("pubmed/")
            .last()
            .unwrap_or(pmid)
            .split('/')
            .next()
            .unwrap_or(pmid)
    } else {
        pmid
    };

    // PMID should be all digits
    pmid.chars().all(|c| c.is_ascii_digit())
}

/// Extract PMID from various formats
pub fn extract_pmid(pmid_input: &str) -> Option<String> {
    if pmid_input.is_empty() {
        return None;
    }

    // Remove "PMID:" prefix if present (case-insensitive)
    let input = pmid_input
        .strip_prefix("PMID:")
        .or_else(|| pmid_input.strip_prefix("pmid:"))
        .unwrap_or(pmid_input);

    // Extract from URL if present
    let input = if input.starts_with("http://") || input.starts_with("https://") {
        // Handle various PubMed URL formats:
        // - https://pubmed.ncbi.nlm.nih.gov/12345678/
        // - https://pubmed.ncbi.nlm.nih.gov/pubmed/12345678
        // - https://www.ncbi.nlm.nih.gov/pubmed/12345678

        // Split by '/' and look for a segment that's all digits
        let parts: Vec<&str> = input.split('/').collect();
        let mut found_digits = None;

        for part in parts {
            // Remove query parameters
            let clean_part = part.split('?').next().unwrap_or(part);

            // Check if this part is all digits (potential PMID)
            if clean_part.chars().all(|c| c.is_ascii_digit()) && !clean_part.is_empty() {
                found_digits = Some(clean_part);
                break;
            }
        }

        found_digits?
    } else {
        input
    };

    // PMID should be all digits
    if input.chars().all(|c| c.is_ascii_digit()) {
        Some(input.to_string())
    } else {
        None
    }
}

/// Fetch metadata for a given PMID using E-utilities API
pub async fn fetch_pubmed_metadata(pmid: &str) -> Result<PubmedMetadata, PubmedError> {
    // Extract and validate PMID
    let extracted_pmid =
        extract_pmid(pmid).ok_or_else(|| PubmedError::InvalidPmid(pmid.to_string()))?;

    // Build the E-utilities EFetch URL
    // NCBI recommends including tool name and email in requests
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&id={}&rettype=xml&retmode=xml&tool=XuanBrain&email=support%40example.com",
        extracted_pmid
    );

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("XuanBrain/0.1.0 (mailto:support@example.com)")
        .build()?;

    // Send request to E-utilities API
    let response = client
        .get(&url)
        .header(ACCEPT, "application/xml")
        .send()
        .await?;

    // Check response status
    let response = response.error_for_status().map_err(|e| {
        if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            PubmedError::NotFound
        } else {
            PubmedError::RequestError(e)
        }
    })?;

    // Parse XML response
    let xml_text = response.text().await?;

    // Check for empty or error response
    if xml_text.contains("<ERROR>") || xml_text.trim().is_empty() {
        return Err(PubmedError::NotFound);
    }

    // Parse the XML
    let article_set: PubmedArticleSet = quick_xml::de::from_str(&xml_text)
        .map_err(|e| PubmedError::XmlError(format!("XML parse error: {}", e)))?;

    // Get the first article
    let article = article_set
        .pubmed_articles
        .into_iter()
        .next()
        .ok_or(PubmedError::NotFound)?;

    article.to_metadata()
}

/// Search PubMed for articles by query
/// Returns a list of PMIDs
pub async fn search_pubmed(query: &str, max_results: u32) -> Result<Vec<String>, PubmedError> {
    // Build the E-utilities ESearch URL
    let url = format!(
        "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retmax={}&retmode=json&tool=XuanBrain&email=support%40example.com",
        urlencoding::encode(query),
        max_results
    );

    // Create HTTP client
    let client = reqwest::Client::builder()
        .user_agent("XuanBrain/0.1.0 (mailto:support@example.com)")
        .build()?;

    // Send request
    let response = client
        .get(&url)
        .header(ACCEPT, "application/json")
        .send()
        .await?;

    let response = response.error_for_status()?;

    // Parse JSON response
    let json: serde_json::Value = response.json().await?;

    // Extract PMIDs from response
    let id_list = json
        .get("esearchresult")
        .and_then(|r| r.get("idlist"))
        .and_then(|ids| ids.as_array())
        .ok_or_else(|| PubmedError::ParseError("Invalid search response".to_string()))?;

    let pmids: Vec<String> = id_list
        .iter()
        .filter_map(|id| id.as_str().map(|s| s.to_string()))
        .collect();

    Ok(pmids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pmid() {
        // Valid PMIDs
        assert_eq!(extract_pmid("12345678"), Some("12345678".to_string()));
        assert_eq!(extract_pmid("PMID:12345678"), Some("12345678".to_string()));
        assert_eq!(extract_pmid("pmid:12345678"), Some("12345678".to_string()));
        assert_eq!(
            extract_pmid("https://pubmed.ncbi.nlm.nih.gov/12345678/"),
            Some("12345678".to_string())
        );
        assert_eq!(
            extract_pmid("https://pubmed.ncbi.nlm.nih.gov/pubmed/12345678"),
            Some("12345678".to_string())
        );

        // Invalid PMIDs
        assert_eq!(extract_pmid(""), None);
        assert_eq!(extract_pmid("abc"), None);
        assert_eq!(extract_pmid("PMID:abc"), None);
    }

    #[test]
    fn test_is_valid_pmid() {
        assert!(is_valid_pmid("12345678"));
        assert!(is_valid_pmid("PMID:12345678"));
        assert!(!is_valid_pmid(""));
        assert!(!is_valid_pmid("abc"));
        assert!(!is_valid_pmid("PMID:abc"));
    }

    #[tokio::test]
    async fn test_fetch_pubmed_metadata() {
        // Using a well-known PMID: 32123456 (COVID-19 related article)
        let pmid = "32123456";

        let result = fetch_pubmed_metadata(pmid).await;

        assert!(
            result.is_ok(),
            "Failed to fetch PubMed metadata: {:?}",
            result
        );

        let metadata = result.unwrap();
        assert_eq!(metadata.pmid, pmid);
        assert!(!metadata.title.is_empty(), "Title should not be empty");
        assert!(!metadata.authors.is_empty(), "Authors should not be empty");

        println!("PMID: {}", metadata.pmid);
        println!("Title: {}", metadata.title);
        println!("Authors: {}", metadata.authors.join(", "));
        println!("Journal: {:?}", metadata.journal_name);
        println!("Year: {:?}", metadata.publication_year);
        println!("DOI: {:?}", metadata.doi);
        if let Some(abstract_text) = &metadata.abstract_text {
            println!(
                "Abstract: {}...",
                abstract_text.chars().take(200).collect::<String>()
            );
        }
        if !metadata.mesh_terms.is_empty() {
            println!("MeSH Terms: {}", metadata.mesh_terms.join(", "));
        }
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_pmid() {
        let result = fetch_pubmed_metadata("99999999999").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_invalid_pmid() {
        let result = fetch_pubmed_metadata("invalid-pmid").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(PubmedError::InvalidPmid(_))));
    }

    #[tokio::test]
    async fn test_search_pubmed() {
        let query = "COVID-19 treatment";
        let result = search_pubmed(query, 5).await;

        assert!(result.is_ok(), "Failed to search PubMed: {:?}", result);

        let pmids = result.unwrap();
        assert!(!pmids.is_empty(), "Should return some PMIDs");
        assert!(pmids.len() <= 5, "Should not exceed max_results");

        println!("Search results for '{}':", query);
        for pmid in &pmids {
            println!("  PMID: {}", pmid);
        }
    }
}
