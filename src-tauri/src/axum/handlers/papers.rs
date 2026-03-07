use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tauri::Emitter;
use tracing::info;
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::models::CreatePaper;
use crate::papers::importer::html::{extract_paper_from_html, HtmlImportError};
use crate::repository::{AuthorRepository, LabelRepository, PaperRepository};
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

// ==================== Zotero Import DTOs ====================

/// Query parameters for Zotero import
#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportZoteroQuery {
    /// Category ID to assign to the imported paper
    pub category_id: Option<String>,
}

/// Zotero creator (author)
#[derive(Debug, Deserialize, ToSchema)]
pub struct ZoteroCreator {
    /// First name / Given name
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    /// Last name / Family name
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    /// Creator type (e.g., "author", "editor")
    #[serde(rename = "creatorType")]
    pub creator_type: Option<String>,
}

/// Zotero attachment
#[derive(Debug, Deserialize, ToSchema)]
pub struct ZoteroAttachment {
    /// Attachment title
    pub title: Option<String>,
    /// Attachment URL
    pub url: Option<String>,
    /// MIME type
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// Zotero tag
#[derive(Debug, Deserialize, ToSchema)]
pub struct ZoteroTag {
    /// Tag name
    pub tag: String,
}

/// Zotero import request body
#[derive(Debug, Deserialize, ToSchema)]
pub struct ImportZoteroRequest {
    /// List of creators (authors, editors, etc.)
    pub creators: Option<Vec<ZoteroCreator>>,
    /// List of attachments
    pub attachments: Option<Vec<ZoteroAttachment>>,
    /// List of tags
    pub tags: Option<Vec<ZoteroTag>>,
    /// Paper title (required)
    pub title: String,
    /// Publication title (journal name)
    #[serde(rename = "publicationTitle")]
    pub publication_title: Option<String>,
    /// Volume
    pub volume: Option<String>,
    /// Issue
    pub issue: Option<String>,
    /// Pages
    pub pages: Option<String>,
    /// Publisher
    pub publisher: Option<String>,
    /// Publication date
    pub date: Option<String>,
    /// DOI
    #[serde(rename = "DOI")]
    pub doi: Option<String>,
    /// ISSN
    #[serde(rename = "ISSN")]
    pub issn: Option<String>,
    /// URL
    pub url: Option<String>,
    /// Abstract
    #[serde(rename = "abstractNote")]
    pub abstract_note: Option<String>,
    /// Language
    pub language: Option<String>,
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
        publisher: None,
        issn: None,
        language: None,
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

/// Import paper from Zotero JSON format
///
/// Creates a new paper record from Zotero JSON format with full metadata support.
/// Supports creators (authors), tags (labels), and all standard bibliographic fields.
///
/// Request body should be JSON in Zotero format.
#[utoipa::path(
    post,
    path = "/api/papers/import-clip",
    tag = "papers",
    params(
        ("category_id" = Option<String>, Query, description = "Category ID to assign to the paper")
    ),
    request_body = ImportZoteroRequest,
    responses(
        (status = 200, description = "Import result", body = ImportHtmlResponse),
        (status = 400, description = "Invalid request or duplicate paper")
    )
)]
pub async fn import_paper_from_zotero(
    State(state): State<AppState>,
    Query(query): Query<ImportZoteroQuery>,
    Json(payload): Json<ImportZoteroRequest>,
) -> Result<Json<ImportHtmlResponse>, ApiError> {
    info!("Importing paper from Zotero JSON via API: {}", payload.title);

    // 1. Check for duplicates by DOI
    if let Some(ref doi) = payload.doi {
        if !doi.is_empty() {
            if let Some(_existing) = PaperRepository::find_by_doi(&state.db, doi)
                .await
                .map_err(ApiError)?
            {
                return Ok(Json(ImportHtmlResponse {
                    success: false,
                    paper: None,
                    error: Some(format!("Paper with DOI {} already exists", doi)),
                }));
            }
        }
    }

    // 2. Check for duplicates by URL
    if let Some(ref url) = payload.url {
        if !url.is_empty() {
            if let Some(_existing) = PaperRepository::find_by_url(&state.db, url)
                .await
                .map_err(ApiError)?
            {
                return Ok(Json(ImportHtmlResponse {
                    success: false,
                    paper: None,
                    error: Some(format!("Paper with URL {} already exists", url)),
                }));
            }
        }
    }

    // 3. Parse publication year from date
    let publication_year = payload
        .date
        .as_ref()
        .and_then(|d| d.split('/').next())
        .and_then(|y| y.parse::<i32>().ok());

    // 4. Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(payload.title.as_bytes());
    let hash_string = format!("{:x}", hasher.finalize());

    // 5. Create paper
    let paper = PaperRepository::create(
        &state.db,
        CreatePaper {
            title: payload.title.clone(),
            doi: payload.doi.filter(|d| !d.is_empty()),
            publication_year,
            publication_date: payload.date.clone(),
            journal_name: payload.publication_title.clone(),
            conference_name: None,
            volume: payload.volume.clone(),
            issue: payload.issue.clone(),
            pages: payload.pages.clone(),
            url: payload.url.filter(|u| !u.is_empty()),
            abstract_text: payload.abstract_note.clone(),
            attachment_path: Some(hash_string),
            publisher: payload.publisher.clone(),
            issn: payload.issn.clone(),
            language: payload.language.clone(),
        },
    )
    .await
    .map_err(ApiError)?;

    let paper_id = paper.id;
    info!("Created paper with id: {}", paper_id);

    // 6. Add authors from creators
    if let Some(ref creators) = payload.creators {
        for (order, creator) in creators.iter().enumerate() {
            // Build full name from first and last name
            let full_name = match (&creator.first_name, &creator.last_name) {
                (Some(first), Some(last)) => {
                    format!("{} {}", first.trim(), last.trim())
                }
                (Some(first), None) => first.trim().to_string(),
                (None, Some(last)) => last.trim().to_string(),
                (None, None) => continue,
            };

            if full_name.is_empty() {
                continue;
            }

            let author = AuthorRepository::create_or_find_from_parts(
                &state.db,
                creator.first_name.as_deref(),
                creator.last_name.as_deref(),
                None,
            )
            .await
            .map_err(ApiError)?;

            PaperRepository::add_author(&state.db, paper_id, author.id, order as i32)
                .await
                .map_err(ApiError)?;
        }
    }

    // 7. Add tags as labels
    if let Some(ref tags) = payload.tags {
        for tag in tags {
            let tag_name = tag.tag.trim();
            if tag_name.is_empty() {
                continue;
            }

            // Find existing label or create new one
            let label = if let Some(existing) = LabelRepository::find_by_name(&state.db, &tag_name)
                .await
                .map_err(ApiError)?
            {
                existing
            } else {
                // Create new label with default color
                LabelRepository::create(
                    &state.db,
                    crate::models::CreateLabel {
                        name: tag_name.to_string(),
                        color: "#607D8B".to_string(),
                    },
                )
                .await
                .map_err(ApiError)?
            };

            // Add label to paper
            LabelRepository::add_to_paper(&state.db, paper_id, label.id)
                .await
                .map_err(ApiError)?;
        }
    }

    // 8. Set category if provided
    if let Some(ref category_id_str) = query.category_id {
        if let Ok(category_id) = category_id_str.parse::<i64>() {
            PaperRepository::set_category(&state.db, paper_id, Some(category_id))
                .await
                .map_err(ApiError)?;
            info!("Assigned paper {} to category {}", paper_id, category_id);
        }
    }

    info!(
        "Successfully imported paper from Zotero: {} (id: {})",
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

    // 10. Build author names for response
    let author_names: Vec<String> = payload
        .creators
        .unwrap_or_default()
        .iter()
        .filter_map(|c| {
            match (&c.first_name, &c.last_name) {
                (Some(first), Some(last)) => Some(format!("{} {}", first.trim(), last.trim())),
                (Some(first), None) => Some(first.trim().to_string()),
                (None, Some(last)) => Some(last.trim().to_string()),
                (None, None) => None,
            }
        })
        .collect();

    // 11. Build tag names for response
    let tag_names: Vec<String> = payload
        .tags
        .unwrap_or_default()
        .iter()
        .map(|t| t.tag.clone())
        .collect();

    // 12. Return response
    Ok(Json(ImportHtmlResponse {
        success: true,
        paper: Some(serde_json::json!({
            "id": paper_id.to_string(),
            "title": paper.title,
            "publication_year": paper.publication_year,
            "publication_date": paper.publication_date,
            "journal_name": paper.journal_name,
            "authors": author_names,
            "tags": tag_names,
            "doi": paper.doi,
            "url": paper.url,
            "abstract": paper.abstract_text,
            "publisher": paper.publisher,
            "issn": paper.issn,
            "language": paper.language,
        })),
        error: None,
    }))
}
