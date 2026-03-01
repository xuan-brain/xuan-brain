//! Search commands using SQLite full-text search
//!
//! These commands use the SQLite repository layer for data access.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::repository::PaperRepository;
use crate::sys::error::Result;

/// Search result with relevance score
#[derive(Serialize, Deserialize)]
pub struct SearchResultDto {
    pub id: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub journal_name: Option<String>,
    pub score: Option<f32>,
}

/// Search papers using SQLite LIKE query
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
            score: None,
        })
        .collect();

    info!("Found {} search results", results.len());
    Ok(results)
}

/// Advanced search with scoring using SQLite FTS (if available)
/// For now, this uses the same simple search
#[tauri::command]
#[instrument(skip(db))]
pub async fn search_papers_with_score(
    db: State<'_, Arc<DatabaseConnection>>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResultDto>> {
    info!("Searching papers with scoring, query: {}", query);

    let _limit = limit.unwrap_or(20);

    // Use simple search for now
    // TODO: Implement SQLite FTS5 for better search with scoring
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
            score: None,
        })
        .collect();

    info!("Found {} search results with scoring", results.len());
    Ok(results)
}
