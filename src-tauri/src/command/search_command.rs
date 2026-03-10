//! Search commands using SQLite FTS5 full-text search
//!
//! These commands use the SQLite FTS5 extension for efficient full-text search
//! with relevance scoring using the BM25 algorithm.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::repository::{PaperRepository, SearchRepository};
use crate::sys::error::Result;

/// Search result with relevance score
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResultDto {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publication_year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub journal_name: Option<String>,
    /// Relevance score (0-100, higher is better)
    pub score: f64,
    /// Labels that matched the search query
    pub matched_labels: Vec<String>,
    /// Attachments that matched the search query
    pub matched_attachments: Vec<String>,
}

/// Search papers using SQLite LIKE query (legacy, kept for compatibility)
#[tauri::command]
#[instrument(skip(db))]
pub async fn search_papers(
    db: State<'_, Arc<DatabaseConnection>>,
    query: String,
) -> Result<Vec<SearchResultDto>> {
    info!("Searching papers with query: {}", query);

    let papers = PaperRepository::search(&db, &query).await?;

    let results: Vec<SearchResultDto> = papers
        .into_iter()
        .map(|p| SearchResultDto {
            id: p.id.to_string(),
            title: p.title,
            abstract_text: p.abstract_text,
            doi: p.doi,
            publication_year: p.publication_year,
            journal_name: p.journal_name,
            score: 0.0, // No score for simple search
            matched_labels: vec![],
            matched_attachments: vec![],
        })
        .collect();

    info!("Found {} search results", results.len());
    Ok(results)
}

/// Full-text search using FTS5 with BM25 relevance scoring
///
/// This is the recommended search method for better results with:
/// - Relevance ranking using BM25 algorithm
/// - Search across title, abstract, labels, and attachments
/// - Chinese text support via unicode61 tokenizer
///
/// # Arguments
/// * `query` - Search query string (supports FTS5 query syntax like AND, OR, NOT)
/// * `limit` - Maximum number of results (default: 50)
#[tauri::command]
#[instrument(skip(db))]
pub async fn search_papers_fts(
    db: State<'_, Arc<DatabaseConnection>>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResultDto>> {
    info!("FTS search with query: '{}'", query);

    // Validate query
    let query = query.trim();
    if query.is_empty() {
        return Ok(vec![]);
    }

    let results = SearchRepository::fts_search(&db, query, limit.map(|l| l as u64)).await?;

    // Convert to DTO
    let dtos: Vec<SearchResultDto> = results
        .into_iter()
        .map(|(paper, score)| {
            // Extract matched labels and attachments from the paper
            // For now, we return all labels/attachments associated with the paper
            // A more sophisticated implementation could highlight which terms matched
            SearchResultDto {
                id: paper.id.to_string(),
                title: paper.title,
                abstract_text: paper.abstract_text,
                doi: paper.doi,
                publication_year: paper.publication_year,
                journal_name: paper.journal_name,
                score,
                matched_labels: vec![], // TODO: Extract from FTS snippet
                matched_attachments: vec![], // TODO: Extract from FTS snippet
            }
        })
        .collect();

    info!("FTS search found {} results", dtos.len());
    Ok(dtos)
}

/// Get search suggestions for autocomplete
///
/// Returns paper titles that start with the given prefix
#[tauri::command]
#[instrument(skip(db))]
pub async fn get_search_suggestions(
    db: State<'_, Arc<DatabaseConnection>>,
    prefix: String,
    limit: Option<i32>,
) -> Result<Vec<String>> {
    info!("Getting search suggestions for prefix: '{}'", prefix);

    let suggestions = SearchRepository::get_search_suggestions(
        &db,
        &prefix,
        limit.unwrap_or(10) as u64,
    )
    .await?;

    info!("Found {} suggestions", suggestions.len());
    Ok(suggestions)
}

/// Rebuild the FTS search index
///
/// This is useful for maintenance or after data corruption
#[tauri::command]
#[instrument(skip(db))]
pub async fn rebuild_search_index(db: State<'_, Arc<DatabaseConnection>>) -> Result<()> {
    info!("Rebuilding search index");

    SearchRepository::rebuild_fts_index(&db).await?;

    info!("Search index rebuilt successfully");
    Ok(())
}

/// Check the FTS index status
///
/// Returns the count of papers in the FTS index
#[tauri::command]
#[instrument(skip(db))]
pub async fn check_fts_index_status(db: State<'_, Arc<DatabaseConnection>>) -> Result<usize> {
    let count = SearchRepository::check_fts_index_status(&db).await?;
    Ok(count)
}

/// Get sample FTS index entries for debugging
///
/// Returns a few entries from the FTS index to verify content
#[tauri::command]
#[instrument(skip(db))]
pub async fn get_fts_sample(
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<Vec<(String, String, String)>> {
    let samples = SearchRepository::get_fts_sample(&db).await?;
    Ok(samples)
}

/// Debug FTS query by testing the raw FTS5 query
///
/// Returns the FTS query string and the number of results found
/// This is useful for debugging FTS search issues
#[tauri::command]
#[instrument(skip(db))]
pub async fn debug_fts_query(
    db: State<'_, Arc<DatabaseConnection>>,
    query: String,
) -> Result<(String, usize, Vec<String>)> {
    use sea_orm::sqlx::{Row, sqlite::SqliteRow};

    info!("Debug FTS query: '{}'", query);

    let sanitized_query = query.replace('\\', "\\\\").replace('"', "\\\"");

    // For trigram tokenizer, simply use the raw query without special handling
    // Trigram automatically handles both Chinese and English by creating 3-character slices
    let fts_query = sanitized_query.clone();

    info!("Debug FTS processed query: '{}'", fts_query);

    // Execute the raw FTS query
    let sql = format!(
        r#"
        SELECT p.title
        FROM paper p
        INNER JOIN (
            SELECT paper_id, -bm25(paper_fts) AS score
            FROM paper_fts
            WHERE paper_fts MATCH '{}'
        ) fts ON p.id = fts.paper_id
        WHERE p.deleted_at IS NULL
        LIMIT 10
        "#,
        fts_query
    );

    let pool = db.get_sqlite_connection_pool();
    let rows: Vec<SqliteRow> = sea_orm::sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|e| crate::sys::error::AppError::generic(format!("FTS debug query failed: {}", e)))?;

    let titles: Vec<String> = rows
        .iter()
        .filter_map(|row| row.try_get::<String, _>(0).ok())
        .collect();

    Ok((fts_query, titles.len(), titles))
}

// ==========================================
// Search History Commands
// ==========================================

use crate::database::entities::search_history;
use crate::repository::SearchHistoryRepository;

/// Search history entry DTO
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchHistoryDto {
    pub id: String,
    pub query: String,
    pub created_at: String,
}

impl From<search_history::Model> for SearchHistoryDto {
    fn from(model: search_history::Model) -> Self {
        SearchHistoryDto {
            id: model.id.to_string(),
            query: model.query,
            created_at: model.created_at.to_rfc3339(),
        }
    }
}
/// Add a search query to history
#[tauri::command]
#[instrument(skip(db))]
pub async fn add_search_history(
    db: State<'_, Arc<DatabaseConnection>>,
    query: String,
) -> Result<SearchHistoryDto> {
    info!("Adding search history: '{}'", query);

    let query = query.trim();
    if query.is_empty() {
        return Err(crate::sys::error::AppError::validation(
            "query",
            "Search query cannot be empty",
        ));
    }

    let result = SearchHistoryRepository::add(&db, &query).await?;

    Ok(SearchHistoryDto::from(result))
}
/// Get recent search history
#[tauri::command]
#[instrument(skip(db))]
pub async fn get_search_history(
    db: State<'_, Arc<DatabaseConnection>>,
    limit: Option<i32>,
) -> Result<Vec<SearchHistoryDto>> {
    info!("Getting search history with limit: {:?}", limit);

    let limit = limit.unwrap_or(20);
    let history = SearchHistoryRepository::get_recent(&db, limit as u64).await?;

    let dtos: Vec<SearchHistoryDto> = history.into_iter().map(SearchHistoryDto::from).collect();

    info!("Found {} search history entries", dtos.len());
    Ok(dtos)
}
/// Clear all search history
#[tauri::command]
#[instrument(skip(db))]
pub async fn clear_search_history(db: State<'_, Arc<DatabaseConnection>>) -> Result<()> {
    info!("Clearing all search history");

    SearchHistoryRepository::clear(&db).await?;

    info!("Search history cleared successfully");
    Ok(())
}
/// Delete a specific search history entry
#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_search_history(
    db: State<'_, Arc<DatabaseConnection>>,
    id: i64,
) -> Result<()> {
    info!("Deleting search history entry with id: {}", id);

    SearchHistoryRepository::delete_by_id(&db, id).await?;

    info!("Search history entry deleted successfully");
    Ok(())
}
