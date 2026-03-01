use axum::{
    body::Bytes,
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use sha1::{Digest, Sha1};
use tauri::Emitter;
use tracing::info;
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::models::CreatePaper;
use crate::papers::importer::html::{extract_paper_from_html, HtmlImportError};
use crate::repository::{AuthorRepository, PaperRepository};
use crate::sys::config::AppConfig;
use crate::sys::error::AppError;

/// List all papers
///
/// Returns a list of all papers in the database with basic metadata.
#[utoipa::path(
    get,
    path = "/api/papers",
    tag = "papers",
    responses(
        (status = 200, description = "List of papers", body = Vec<serde_json::Value>)
    )
)]
pub async fn list_papers(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    let papers = PaperRepository::find_all(&state.db).await.map_err(ApiError)?;

    let result: Vec<serde_json::Value> = papers
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id.to_string(),
                "title": p.title,
                "abstract": p.abstract_text,
                "doi": p.doi,
                "publication_year": p.publication_year,
                "journal_name": p.journal_name,
                "url": p.url,
                "read_status": p.read_status,
            })
        })
        .collect();

    Ok(Json(result))
}

/// Get a paper by ID
///
/// Returns detailed information about a specific paper including notes and read status.
#[utoipa::path(
    get,
    path = "/api/papers/{id}",
    tag = "papers",
    params(
        ("id" = String, Path, description = "Paper ID")
    ),
    responses(
        (status = 200, description = "Paper details", body = serde_json::Value),
        (status = 404, description = "Paper not found")
    )
)]
pub async fn get_paper(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let paper_id = id.parse::<i64>()
        .map_err(|_| ApiError(AppError::validation("id", "Invalid paper id format")))?;

    let paper = PaperRepository::find_by_id(&state.db, paper_id).await.map_err(ApiError)?;

    match paper {
        Some(p) => Ok(Json(serde_json::json!({
            "id": p.id.to_string(),
            "title": p.title,
            "abstract": p.abstract_text,
            "doi": p.doi,
            "publication_year": p.publication_year,
            "journal_name": p.journal_name,
            "url": p.url,
            "notes": p.notes,
            "read_status": p.read_status,
        }))),
        None => Err(ApiError(AppError::not_found("Paper", id))),
    }
}

/// Response for HTML import
#[derive(Serialize, ToSchema)]
pub struct ImportHtmlResponse {
    /// Whether the import was successful
    pub success: bool,
    /// Imported paper data (if successful)
    pub paper: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Import paper from HTML content using AI
///
/// Extracts paper metadata from HTML content using AI and creates a new paper record.
/// The endpoint uses configured LLM providers to parse title, authors, DOI, and other metadata.
///
/// Request body should be raw HTML content (text/html or text/plain).
#[utoipa::path(
    post,
    path = "/api/papers/import-html",
    tag = "papers",
    request_body(
        content = String,
        content_type = "text/html"
    ),
    responses(
        (status = 200, description = "Import result", body = ImportHtmlResponse),
        (status = 400, description = "Invalid request or no LLM provider configured")
    )
)]
pub async fn import_paper_from_html(
    State(state): State<AppState>,
    body: Bytes,
) -> Result<Json<ImportHtmlResponse>, ApiError> {
    info!("Importing paper from HTML via API");

    let html = String::from_utf8(body.to_vec()).map_err(|e| {
        ApiError(AppError::validation(
            "html",
            format!("Invalid UTF-8 content: {}", e),
        ))
    })?;

    // 1. Load config to get LLM provider
    let config = AppConfig::load(&state.app_dirs.config).map_err(|e| {
        ApiError(AppError::config_error(
            "settings.json",
            e.to_string(),
        ))
    })?;

    // 2. Find default or first LLM provider
    let provider = config
        .system
        .llm_providers
        .iter()
        .find(|p| p.is_default)
        .or_else(|| config.system.llm_providers.first())
        .ok_or_else(|| {
            ApiError(AppError::validation(
                "llm_provider",
                "No LLM provider configured. Please add an LLM provider in settings.",
            ))
        })?;

    // 3. Extract metadata from HTML using AI
    let metadata = match extract_paper_from_html(&html, provider).await {
        Ok(m) => {
            info!("Extracted metadata from LLM: {:?}", m);
            m
        }
        Err(HtmlImportError::AiError(msg)) => {
            return Ok(Json(ImportHtmlResponse {
                success: false,
                paper: None,
                error: Some(msg),
            }));
        }
        Err(HtmlImportError::MissingTitle) => {
            return Ok(Json(ImportHtmlResponse {
                success: false,
                paper: None,
                error: Some("Could not find paper title in the provided HTML".to_string()),
            }));
        }
        Err(e) => {
            return Ok(Json(ImportHtmlResponse {
                success: false,
                paper: None,
                error: Some(format!("Failed to extract metadata: {}", e)),
            }));
        }
    };

    // 4. Check for duplicates by DOI
    if let Some(ref doi) = metadata.doi {
        if !doi.is_empty() {
            if let Some(_existing) = PaperRepository::find_by_doi(&state.db, doi).await.map_err(ApiError)? {
                return Ok(Json(ImportHtmlResponse {
                    success: false,
                    paper: None,
                    error: Some(format!("Paper with DOI {} already exists", doi)),
                }));
            }
        }
    }

    // 5. Check for duplicates by URL
    if let Some(ref url) = metadata.url {
        if !url.is_empty() {
            if let Some(_existing) = PaperRepository::find_by_url(&state.db, url).await.map_err(ApiError)? {
                return Ok(Json(ImportHtmlResponse {
                    success: false,
                    paper: None,
                    error: Some(format!("Paper with URL {} already exists", url)),
                }));
            }
        }
    }

    // 6. Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(metadata.title.as_bytes());
    let hash_string = format!("{:x}", hasher.finalize());

    // 7. Create paper
    let paper = PaperRepository::create(&state.db, CreatePaper {
        title: metadata.title.clone(),
        doi: metadata.doi.filter(|d| !d.is_empty()),
        publication_year: metadata.publication_year.and_then(|y| i32::try_from(y).ok()),
        publication_date: None,
        journal_name: metadata.journal_name,
        conference_name: None,
        volume: metadata.volume,
        issue: metadata.issue,
        pages: metadata.pages,
        url: metadata.url.filter(|u| !u.is_empty()),
        abstract_text: metadata.abstract_text,
        attachment_path: Some(hash_string),
    }).await.map_err(ApiError)?;

    let paper_id = paper.id;
    info!("Created paper with id: {}", paper_id);

    // 8. Add authors
    for (order, author_name) in metadata.authors.iter().enumerate() {
        if author_name.trim().is_empty() {
            continue;
        }

        let author = AuthorRepository::create_or_find(&state.db, author_name.trim(), None).await.map_err(ApiError)?;
        let author_id = author.id;
        PaperRepository::add_author(&state.db, paper_id, author_id, order as i32).await.map_err(ApiError)?;
    }

    info!(
        "Successfully imported paper from HTML: {} (id: {})",
        paper.title, paper_id
    );

    // 9. Emit event to notify frontend to refresh paper list
    if let Some(app_handle) = &state.app_handle {
        let _ = app_handle.emit(
            "paper:imported",
            serde_json::json!({
                "id": paper_id.to_string(),
                "title": paper.title,
            }),
        );
        info!("Emitted paper:imported event for paper id: {}", paper_id);
    }

    // 10. Return response
    Ok(Json(ImportHtmlResponse {
        success: true,
        paper: Some(serde_json::json!({
            "id": paper_id.to_string(),
            "title": paper.title,
            "publication_year": paper.publication_year,
            "journal_name": paper.journal_name,
            "authors": metadata.authors,
            "doi": paper.doi,
            "url": paper.url,
            "abstract": paper.abstract_text,
        })),
        error: None,
    }))
}
