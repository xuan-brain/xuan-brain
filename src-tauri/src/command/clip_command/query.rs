//! Query commands for clip operations

use std::sync::Arc;

use tauri::State;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::repository::ClippingRepository;
use crate::sys::error::{AppError, Result};

use super::dtos::{ClipDto, CommentDto};

/// Convert Clipping comments to CommentDto
fn comments_to_dto(
    comments: Vec<crate::models::Comment>,
) -> Vec<CommentDto> {
    comments
        .into_iter()
        .map(|c| CommentDto {
            id: c.id.to_string(),
            clipping_id: c.clipping_id.to_string(),
            content: c.content,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect()
}

/// List all clips with optional pagination
#[tauri::command]
#[instrument(skip(db))]
pub async fn list_clips(
    db: State<'_, Arc<DatabaseConnection>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipDto>> {
    info!("Fetching clips (limit: {:?}, offset: {:?})", limit, offset);

    let clippings = ClippingRepository::get_all_clippings(&db).await?;

    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(clippings.len());

    let mut result = Vec::new();
    for c in clippings.into_iter().skip(offset_val).take(limit_val) {
        // Get comments for this clipping
        let comments = ClippingRepository::get_comments(&db, c.id).await.unwrap_or_default();
        result.push(ClipDto {
            id: c.id.to_string(),
            title: c.title,
            url: c.url,
            content: c.content,
            source_domain: c.source_domain,
            author: c.author,
            published_date: c.published_date,
            excerpt: c.excerpt,
            thumbnail_url: c.thumbnail_url,
            read_status: c.read_status,
            notes: c.notes,
            tags: c.tags,
            image_paths: c.image_paths,
            comments: comments_to_dto(comments),
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        });
    }

    info!("Fetched {} clips", result.len());
    Ok(result)
}

/// Get a single clip by ID
#[tauri::command]
#[instrument(skip(db))]
pub async fn get_clip(id: String, db: State<'_, Arc<DatabaseConnection>>) -> Result<Option<ClipDto>> {
    info!("Fetching clip with id: {}", id);

    let clip_id = id.parse::<i64>()
        .map_err(|_| AppError::validation("id", "Invalid clip id format"))?;

    let clipping = ClippingRepository::get_clipping_by_id(&db, clip_id).await?;

    match clipping {
        Some(c) => {
            info!("Found clip: {}", id);
            // Get comments for this clipping
            let comments = ClippingRepository::get_comments(&db, c.id).await.unwrap_or_default();
            Ok(Some(ClipDto {
                id: c.id.to_string(),
                title: c.title,
                url: c.url,
                content: c.content,
                source_domain: c.source_domain,
                author: c.author,
                published_date: c.published_date,
                excerpt: c.excerpt,
                thumbnail_url: c.thumbnail_url,
                read_status: c.read_status,
                notes: c.notes,
                tags: c.tags,
                image_paths: c.image_paths,
                comments: comments_to_dto(comments),
                created_at: c.created_at.to_rfc3339(),
                updated_at: c.updated_at.to_rfc3339(),
            }))
        }
        None => {
            info!("Clip not found: {}", id);
            Ok(None)
        }
    }
}
