//! Search commands using SurrealDB BM25 full-text search
//!
//! These commands use the SurrealDB repository layer for data access.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordIdKey, SurrealValue};
use tauri::State;
use tracing::{info, instrument};

use crate::repository::PaperRepository;
use crate::surreal::connection::SurrealClient;
use crate::sys::error::Result;

/// Convert RecordIdKey to string
fn record_id_key_to_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        RecordIdKey::Array(_) => "array".to_string(),
        RecordIdKey::Object(_) => "object".to_string(),
        RecordIdKey::Range(_) => "range".to_string(),
    }
}

/// Search result with relevance score
#[derive(Serialize, Deserialize, SurrealValue)]
pub struct SearchResultDto {
    pub id: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i32>,
    pub journal_name: Option<String>,
    pub score: Option<f32>,
}

/// Search papers using SurrealDB BM25 full-text search
#[tauri::command]
#[instrument(skip(db))]
pub async fn search_papers(
    db: State<'_, Arc<SurrealClient>>,
    query: String,
) -> Result<Vec<SearchResultDto>> {
    info!("Searching papers with query: {}", query);

    let repo = PaperRepository::new(&db);
    let papers = repo.search(&query).await?;

    let results: Vec<SearchResultDto> = papers
        .into_iter()
        .map(|p| SearchResultDto {
            id: p.id.map(|rid| format!("{}:{}", rid.table, record_id_key_to_string(&rid.key))).unwrap_or_default(),
            title: p.title,
            abstract_text: p.abstract_text,
            doi: p.doi,
            publication_year: p.publication_year,
            journal_name: p.journal_name,
            score: None, // Basic search doesn't return score
        })
        .collect();

    info!("Found {} search results", results.len());
    Ok(results)
}

/// Advanced search with scoring using SurrealDB BM25
#[tauri::command]
#[instrument(skip(db))]
pub async fn search_papers_with_score(
    db: State<'_, Arc<SurrealClient>>,
    query: String,
    limit: Option<i32>,
) -> Result<Vec<SearchResultDto>> {
    info!("Searching papers with scoring, query: {}", query);

    let limit = limit.unwrap_or(20);

    // Use SurrealDB's BM25 scoring
    let results: Vec<SearchResultDto> = db
        .query(
            r#"
            SELECT
                id,
                title,
                `abstract` AS abstract_text,
                doi,
                publication_year,
                journal_name,
                search::score(0) AS title_score,
                search::score(1) AS abstract_score
            FROM paper
            WHERE deleted_at IS NONE
            AND (title @0@ $query OR `abstract` @1@ $query)
            ORDER BY title_score DESC, abstract_score DESC
            LIMIT $limit
            "#,
        )
        .bind(("query", query.clone()))
        .bind(("limit", limit))
        .await
        .map_err(|e| crate::sys::error::AppError::surrealdb_error("search", e.to_string()))?
        .take(0)
        .map_err(|e| crate::sys::error::AppError::surrealdb_error("search_results", e.to_string()))?;

    info!("Found {} search results with scoring", results.len());
    Ok(results)
}
