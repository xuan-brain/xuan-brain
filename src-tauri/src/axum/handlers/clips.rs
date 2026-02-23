//! API handlers for clipping operations

use std::fs;
use std::path::PathBuf;

use ammonia::clean;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use tracing::{info, warn};
use utoipa::ToSchema;

use crate::axum::error::ApiError;
use crate::axum::state::AppState;
use crate::repository::ClippingRepository;
use crate::surreal::models::{Clipping, CreateClipping, UpdateClipping};
use crate::sys::error::AppError;

/// Convert RecordId to string helper
fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    use surrealdb_types::RecordIdKey;
    format!("{}:{}", id.table, match &id.key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        _ => "unknown".to_string(),
    })
}

/// Query parameters for list_clips endpoint
#[derive(Debug, Deserialize, ToSchema)]
pub struct ListClipsQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Response for clipping endpoints
#[derive(serde::Serialize, ToSchema)]
pub struct ClippingResponse {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: i32,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub image_paths: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Clipping> for ClippingResponse {
    fn from(clipping: Clipping) -> Self {
        Self {
            id: clipping.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
            title: clipping.title,
            url: clipping.url,
            content: clipping.content,
            source_domain: clipping.source_domain,
            author: clipping.author,
            published_date: clipping.published_date.map(|d| d.to_rfc3339()),
            excerpt: clipping.excerpt,
            thumbnail_url: clipping.thumbnail_url,
            read_status: clipping.read_status,
            notes: clipping.notes,
            tags: clipping.tags,
            image_paths: clipping.image_paths,
            created_at: clipping.created_at.to_rfc3339(),
            updated_at: clipping.updated_at.to_rfc3339(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/clips",
    tag = "clips",
    params(
        ("limit" = Option<usize>, Query, description = "Maximum number of results to return"),
        ("offset" = Option<usize>, Query, description = "Number of results to skip")
    ),
    responses(
        (status = 200, description = "List of clippings", body = Vec<ClippingResponse>)
    )
)]
#[instrument(skip(state))]
pub async fn list_clips(
    State(state): State<AppState>,
    Query(params): Query<ListClipsQuery>,
) -> Result<Json<Vec<ClippingResponse>>, ApiError> {
    let clippings = ClippingRepository::get_all_clippings(&state.db).await.map_err(ApiError)?;
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(clippings.len());
    let result: Vec<ClippingResponse> = clippings
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(ClippingResponse::from)
        .collect();
    Ok(Json(result))
}

#[utoipa::path(
    get,
    path = "/api/clips/{id}",
    tag = "clips",
    params(
        ("id" = String, Path, description = "Clipping ID (e.g., 'clipping:abc123')")
    ),
    responses(
        (status = 200, description = "Clipping details", body = ClippingResponse),
        (status = 404, description = "Clipping not found")
    )
)]
#[instrument(skip(state))]
pub async fn get_clip(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClippingResponse>, ApiError> {
    let clipping = ClippingRepository::get_clipping_by_id(&state.db, &id).await.map_err(ApiError)?;
    match clipping {
        Some(c) => Ok(Json(ClippingResponse::from(c))),
        None => Err(ApiError(AppError::not_found("Clipping", id))),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateClippingRequest {
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateClippingResponse {
    pub id: String,
    pub title: String,
    pub url: String,
    pub content: String,
    pub source_domain: String,
    pub image_paths: Vec<String>,
}

async fn download_image(url: &str, clip_id: &str, files_dir: &str) -> Result<String, AppError> {
    let filename = extract_filename_from_url(url);
    let clip_dir = PathBuf::from(files_dir).join("clips").join(clip_id).join("images");
    fs::create_dir_all(&clip_dir).map_err(|e| {
        AppError::file_system(clip_dir.display().to_string(), format!("Failed to create images directory: {}", e))
    })?;
    let response = reqwest::get(url).await.map_err(|e| {
        AppError::generic(format!("Failed to download image from {}: {}", url, e))
    })?;
    if !response.status().is_success() {
        return Err(AppError::generic(format!("Failed to download image from {}: HTTP {}", url, response.status())));
    }
    let bytes = response.bytes().await.map_err(|e| {
        AppError::generic(format!("Failed to read image bytes from {}: {}", url, e))
    })?;
    let local_path = clip_dir.join(&filename);
    fs::write(&local_path, bytes).map_err(|e| {
        AppError::file_system(local_path.display().to_string(), format!("Failed to write image: {}", e))
    })?;
    info!("Downloaded image from {} to {:?}", url, local_path);
    Ok(format!("/clips/images/{}/images/{}", clip_id, filename))
}

fn extract_filename_from_url(url: &str) -> String {
    if let Some(parsed) = url.split('?').next() {
        if let Some(filename) = parsed.split('/').last() {
            if !filename.is_empty() {
                if filename.contains('.') {
                    return filename.to_string();
                }
                use sha1::{Digest, Sha1};
                let mut hasher = Sha1::new();
                hasher.update(url.as_bytes());
                return format!("{:x}.jpg", hasher.finalize());
            }
        }
    }
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    format!("{:x}.jpg", hasher.finalize())
}

async fn process_markdown_images(
    content: String,
    clip_id: &str,
    files_dir: &str,
) -> Result<(String, Vec<String>), AppError> {
    let image_regex = Regex::new(r"!\[.*?\]\((https?://[^\)]+)\)")
        .map_err(|e| AppError::generic(format!("Failed to compile regex: {}", e)))?;
    let mut updated_content = content.clone();
    let mut image_paths = Vec::new();
    let mut offset: i64 = 0;
    for cap in image_regex.captures_iter(content.as_str()) {
        if let Some(url_match) = cap.get(1) {
            let url = url_match.as_str();
            let start = url_match.start();
            let end = url_match.end();
            match download_image(url, clip_id, files_dir).await {
                Ok(local_path) => {
                    let adjusted_start = start + offset as usize;
                    let adjusted_end = end + offset as usize;
                    updated_content.replace_range(adjusted_start..adjusted_end, &local_path);
                    offset += local_path.len() as i64 - (end - start) as i64;
                    image_paths.push(local_path);
                }
                Err(e) => {
                    warn!("Failed to download image from {}: {}, skipping", url, e);
                }
            }
        }
    }
    Ok((updated_content, image_paths))
}

#[utoipa::path(
    post,
    path = "/api/clips",
    tag = "clips",
    request_body = CreateClippingRequest,
    responses(
        (status = 201, description = "Clipping created successfully", body = CreateClippingResponse),
        (status = 400, description = "Invalid request data"),
        (status = 500, description = "Internal server error")
    )
)]
#[instrument(skip(state))]
pub async fn create_clip(
    State(state): State<AppState>,
    Json(payload): Json<CreateClippingRequest>,
) -> Result<(StatusCode, Json<CreateClippingResponse>), ApiError> {
    info!("Creating clipping: {}", payload.title);
    let sanitized_content = clean(&payload.content);
    let create_clipping = CreateClipping {
        title: payload.title.clone(),
        url: payload.url.clone(),
        content: sanitized_content.clone(),
        source_domain: payload.source_domain.clone(),
        author: payload.author.clone(),
        published_date: payload.published_date.as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        excerpt: payload.excerpt.clone(),
        thumbnail_url: payload.thumbnail_url.clone(),
        tags: payload.tags.clone(),
        image_paths: Vec::new(),
    };
    let clipping = ClippingRepository::create_clipping(&state.db, create_clipping).await.map_err(ApiError)?;
    let clip_id = clipping.id.as_ref()
        .map(record_id_to_string)
        .and_then(|s| s.split(':').last().map(String::from))
        .unwrap_or_else(|| chrono::Utc::now().timestamp().to_string());
    let (processed_content, image_paths) = process_markdown_images(sanitized_content, &clip_id, &state.app_dirs.files)
        .await
        .map_err(|e| {
            AppError::file_system(&clip_id, format!("Failed to process images: {}", e))
        })?;
    let update_clipping = UpdateClipping {
        title: None,
        url: None,
        content: Some(processed_content.clone()),
        source_domain: None,
        author: None,
        published_date: None,
        excerpt: None,
        thumbnail_url: None,
        read_status: None,
        notes: None,
        tags: None,
        image_paths: Some(image_paths.clone()),
    };
    let updated = ClippingRepository::update_clipping(&state.db, &clip_id, update_clipping).await.map_err(ApiError)?;
    if updated.is_none() {
        return Err(ApiError(AppError::generic("Failed to update clipping with image paths".to_string())));
    }
    info!("Successfully created clipping {} with {} images", clip_id, image_paths.len());
    Ok((StatusCode::CREATED, Json(CreateClippingResponse {
        id: clip_id,
        title: payload.title,
        url: payload.url,
        content: processed_content,
        source_domain: payload.source_domain,
        image_paths,
    })))
}
