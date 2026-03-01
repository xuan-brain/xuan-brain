//! Mutation commands for clip operations

use std::sync::Arc;

use ammonia::clean;
use tauri::{AppHandle, State};
use tracing::{info, instrument, warn};

use crate::database::DatabaseConnection;
use crate::models::{CreateClipping, UpdateClipping};
use crate::repository::ClippingRepository;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::{CommentDto, CreateClipRequest, CreateClipResponse};
use super::utils::process_markdown_images;

/// Create a new clip with image downloading
#[tauri::command]
#[instrument(skip(db, app_dirs, _app))]
pub async fn create_clip(
    _app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
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
        content: Some(sanitized_content.clone()),
        source_domain: Some(payload.source_domain.clone()),
        author: payload.author.clone(),
        published_date: payload.published_date.clone(),
        excerpt: payload.excerpt.clone(),
        thumbnail_url: payload.thumbnail_url.clone(),
        tags: payload.tags.clone(),
        image_paths: Vec::new(),
    };

    let clipping = ClippingRepository::create_clipping(&db, create_clipping).await?;

    // Extract clip ID for image processing
    let clip_id = clipping.id.to_string();

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
    };

    let updated = ClippingRepository::update_clipping(&db, clipping.id, update_clipping).await?;

    if updated.is_none() {
        warn!("Failed to update clipping with image paths, but clip was created");
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
        content: Some(processed_content),
        source_domain: Some(payload.source_domain),
        image_paths,
    })
}

/// Add a comment to a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn add_clip_comment(
    db: State<'_, Arc<DatabaseConnection>>,
    clip_id: String,
    content: String,
) -> Result<CommentDto> {
    info!("Adding comment to clip: {}", clip_id);

    let clip_id_num = clip_id.parse::<i64>()
        .map_err(|_| AppError::validation("clip_id", "Invalid clip id format"))?;

    let comment = ClippingRepository::add_comment(&db, clip_id_num, &content).await?;

    Ok(CommentDto {
        id: comment.id.to_string(),
        clipping_id: comment.clipping_id.to_string(),
        content: comment.content,
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    })
}

/// Update a comment in a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn update_clip_comment(
    db: State<'_, Arc<DatabaseConnection>>,
    _clip_id: String,
    comment_id: String,
    content: String,
) -> Result<CommentDto> {
    info!("Updating comment {} in clip: {}", comment_id, _clip_id);

    let comment_id_num = comment_id.parse::<i64>()
        .map_err(|_| AppError::validation("comment_id", "Invalid comment id format"))?;

    let comment = ClippingRepository::update_comment(&db, comment_id_num, &content).await?;

    Ok(CommentDto {
        id: comment.id.to_string(),
        clipping_id: comment.clipping_id.to_string(),
        content: comment.content,
        created_at: comment.created_at.to_rfc3339(),
        updated_at: comment.updated_at.to_rfc3339(),
    })
}

/// Delete a comment from a clip
#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_clip_comment(
    db: State<'_, Arc<DatabaseConnection>>,
    _clip_id: String,
    comment_id: String,
) -> Result<()> {
    info!("Deleting comment {} from clip: {}", comment_id, _clip_id);

    let comment_id_num = comment_id.parse::<i64>()
        .map_err(|_| AppError::validation("comment_id", "Invalid comment id format"))?;

    ClippingRepository::delete_comment(&db, comment_id_num).await?;

    info!(
        "Successfully deleted comment {} from clip {}",
        comment_id, _clip_id
    );
    Ok(())
}
