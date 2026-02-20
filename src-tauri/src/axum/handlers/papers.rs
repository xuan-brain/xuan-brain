use axum::{
    body::Bytes,
    extract::{Path, State},
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tauri::Emitter;
use tracing::info;
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::database::entities::{authors, paper_authors, paper_category, papers, prelude::*};
use crate::papers::importer::html::{extract_paper_from_html, HtmlImportError};
use crate::sys::config::AppConfig;

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
    let db = &*state.db;
    let papers = papers::Entity::find()
        .all(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    let result: Vec<serde_json::Value> = papers
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title,
                "abstract": p.r#abstract,
                "doi": p.doi,
                "publication_year": p.publication_year,
                "journal_name": p.journal_name,
                "url": p.url,
                "read_status": p.read_status,
                "created_at": p.created_at,
                "updated_at": p.updated_at
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
        ("id" = i64, Path, description = "Paper ID")
    ),
    responses(
        (status = 200, description = "Paper details", body = serde_json::Value),
        (status = 404, description = "Paper not found")
    )
)]
pub async fn get_paper(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = &*state.db;
    let paper = papers::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    match paper {
        Some(p) => Ok(Json(serde_json::json!({
            "id": p.id,
            "title": p.title,
            "abstract": p.r#abstract,
            "doi": p.doi,
            "publication_year": p.publication_year,
            "journal_name": p.journal_name,
            "url": p.url,
            "notes": p.notes,
            "read_status": p.read_status,
            "created_at": p.created_at,
            "updated_at": p.updated_at
        }))),
        None => Err(ApiError(crate::sys::error::AppError::NotFound {
            resource_type: "Paper".to_string(),
            resource_id: id.to_string(),
        })),
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
        ApiError(crate::sys::error::AppError::validation(
            "html",
            format!("Invalid UTF-8 content: {}", e),
        ))
    })?;

    let db = &*state.db;

    // 1. Load config to get LLM provider
    let config = AppConfig::load(&state.app_dirs.config).map_err(|e| {
        ApiError(crate::sys::error::AppError::config_error(
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
            ApiError(crate::sys::error::AppError::validation(
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
            if let Some(_existing) = Papers::find()
                .filter(papers::Column::Doi.eq(doi))
                .one(db)
                .await
                .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?
            {
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
            if let Some(_existing) = Papers::find()
                .filter(papers::Column::Url.eq(url))
                .one(db)
                .await
                .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?
            {
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
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(metadata.doi.filter(|d| !d.is_empty())),
        publication_year: Set(metadata.publication_year),
        journal_name: Set(metadata.journal_name),
        url: Set(metadata.url.filter(|u| !u.is_empty())),
        r#abstract: Set(metadata.abstract_text),
        volume: Set(metadata.volume),
        issue: Set(metadata.issue),
        pages: Set(metadata.pages),
        attachment_path: Set(Some(hash_string)),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;

    info!("Created paper with id: {}", paper.id);

    // 8. Add authors
    for author_name in &metadata.authors {
        if author_name.trim().is_empty() {
            continue;
        }

        // Find or create author
        let author = if let Some(existing) = Authors::find()
            .filter(authors::Column::Name.eq(author_name.trim()))
            .one(db)
            .await
            .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?
        {
            existing
        } else {
            authors::ActiveModel {
                name: Set(author_name.trim().to_string()),
                ..Default::default()
            }
            .insert(db)
            .await
            .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db)
        .await
        .map_err(|e| ApiError(crate::sys::error::AppError::SeaOrmError(e)))?;
    }

    info!(
        "Successfully imported paper from HTML: {} (id: {})",
        paper.title, paper.id
    );

    // 9. Emit event to notify frontend to refresh paper list
    if let Some(app_handle) = &state.app_handle {
        let _ = app_handle.emit("paper:imported", serde_json::json!({
            "id": paper.id,
            "title": paper.title,
        }));
        info!("Emitted paper:imported event for paper id: {}", paper.id);
    }

    // 10. Return response
    Ok(Json(ImportHtmlResponse {
        success: true,
        paper: Some(serde_json::json!({
            "id": paper.id,
            "title": paper.title,
            "publication_year": paper.publication_year,
            "journal_name": paper.journal_name,
            "authors": metadata.authors,
            "doi": paper.doi,
            "url": paper.url,
            "abstract": paper.r#abstract,
        })),
        error: None,
    }))
}
