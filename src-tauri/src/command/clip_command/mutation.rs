//! Mutation commands for clip operations

use std::sync::Arc;

use ammonia::clean;
use chrono::Utc;
use tauri::{AppHandle, State};
use tracing::{info, instrument, warn};

use crate::repository::ClippingRepository;
use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateClipping, UpdateClipping};
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::{CommentDto, CreateClipRequest, CreateClipResponse};
use super::utils::{process_markdown_images, record_id_to_string};

/// Create a new clip with image downloading
#[tauri::command]
#[instrument(skip(db, app_dirs, _app))]
pub async fn create_clip(
    _app: AppHandle,
    db: State<'_, Arc<SurrealClient>>,
    app_dirs: State<'_, AppDirs>,
    payload: CreateClipRequest,
) -> Result<CreateClipResponse> {
    info!("Creating clip: {}", payload.title);

    // Sanitize HTML content
    let sanitized_content = clean(&payload.content);

    // Create initial clipping record
    let create_clipping = CreateClipping {
        title: payload.title.clone(),
        url: payload.url.clone(),
        content: sanitized_content.clone(),
        source_domain: payload.source_domain.clone(),
        author: payload.author.clone(),
        published_date: payload
            .published_date
            .as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        excerpt: payload.excerpt.clone(),
        thumbnail_url: payload.thumbnail_url.clone(),
        tags: payload.tags.clone(),
        image_paths: Vec::new(),
        comments: Vec::new(),
    };

    let clipping = ClippingRepository::create_clipping(&db, create_clipping).await?;

    // Extract clip ID for image processing
    let clip_id = clipping
        .id
        .as_ref()
        .map(record_id_to_string)
        .and_then(|s| s.split(':').next_back().map(String::from))
        .unwrap_or_else(|| Utc::now().timestamp().to_string());

    // Process and download images
    let (processed_content, image_paths) =
        process_markdown_images(sanitized_content, &clip_id, &app_dirs.files)
            .await
            .map_err(|e| {
                AppError::file_system(&clip_id, format!("Failed to process images: {}", e))
            })?;

    // Update clipping with processed content and image paths
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
        comments: None,
    };

    let updated = ClippingRepository::update_clipping(&db, &clip_id, update_clipping).await?;

    if updated.is_none() {
        warn!(
            "Failed to update clipping with image paths, but clip was created"
        );
    }

    info!(
        "Successfully created clip {} with {} images",
        clip_id,
        image_paths.len()
    );

    Ok(CreateClipResponse {
        id: clip_id,
        title: payload.title,
        url: payload.url,
        content: processed_content,
        source_domain: payload.source_domain,
        image_paths,
    })
}

/// Add a comment to a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn add_clip_comment(
    db: State<'_, Arc<SurrealClient>>,
    clipId: String,
    content: String,
) -> Result<CommentDto> {
    info!("Adding comment to clip: {}", clipId);

    let updated = ClippingRepository::add_comment(&db, &clipId, &content).await?;

    // Find the newly added comment (last one in the array)
    let comment = updated
        .comments
        .last()
        .ok_or_else(|| AppError::generic("Failed to get added comment".to_string()))?;

    Ok(CommentDto {
        id: comment.id.clone(),
        content: comment.content.clone(),
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    })
}

/// Update a comment in a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn update_clip_comment(
    db: State<'_, Arc<SurrealClient>>,
    clipId: String,
    commentId: String,
    content: String,
) -> Result<CommentDto> {
    info!("Updating comment {} in clip: {}", commentId, clipId);

    let updated =
        ClippingRepository::update_comment(&db, &clipId, &commentId, &content).await?;

    // Find the updated comment
    let comment = updated
        .comments
        .iter()
        .find(|c| c.id == commentId)
        .ok_or_else(|| AppError::not_found("Comment", commentId.to_string()))?;

    Ok(CommentDto {
        id: comment.id.clone(),
        content: comment.content.clone(),
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    })
}

/// Delete a comment from a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_clip_comment(
    db: State<'_, Arc<SurrealClient>>,
    clipId: String,
    commentId: String,
) -> Result<()> {
    info!("Deleting comment {} from clip: {}", commentId, clipId);

    ClippingRepository::delete_comment(&db, &clipId, &commentId).await?;

    info!("Successfully deleted comment {} from clip {}", commentId, clipId);
    Ok(())
}
