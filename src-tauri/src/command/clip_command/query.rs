//! Query commands for clip operations

use std::sync::Arc;

use tauri::State;
use tracing::{info, instrument};

use crate::repository::ClippingRepository;
use crate::surreal::connection::SurrealClient;
use crate::sys::error::Result;

use super::dtos::ClipDto;
use super::utils::record_id_to_string;

/// List all clips with optional pagination
#[tauri::command]
#[instrument(skip(db))]
pub async fn list_clips(
    db: State<'_, Arc<SurrealClient>>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ClipDto>> {
    info!("Fetching clips (limit: {:?}, offset: {:?})", limit, offset);

    let clippings = ClippingRepository::get_all_clippings(&db).await?;

    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(clippings.len());

    let result: Vec<ClipDto> = clippings
        .into_iter()
        .skip(offset_val)
        .take(limit_val)
        .map(|c| ClipDto {
            id: c.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
            title: c.title,
            url: c.url,
            content: c.content,
            source_domain: c.source_domain,
            author: c.author,
            published_date: c.published_date.map(|d| d.to_rfc3339()),
            excerpt: c.excerpt,
            thumbnail_url: c.thumbnail_url,
            read_status: c.read_status,
            notes: c.notes,
            tags: c.tags,
            image_paths: c.image_paths,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        })
        .collect();

    info!("Fetched {} clips", result.len());
    Ok(result)
}

/// Get a single clip by ID
#[tauri::command]
#[instrument(skip(db))]
pub async fn get_clip(id: String, db: State<'_, Arc<SurrealClient>>) -> Result<Option<ClipDto>> {
    info!("Fetching clip with id: {}", id);

    let clipping = ClippingRepository::get_clipping_by_id(&db, &id).await?;

    match clipping {
        Some(c) => {
            info!("Found clip: {}", id);
            Ok(Some(ClipDto {
                id: c.id.map(|rid| record_id_to_string(&rid)).unwrap_or_default(),
                title: c.title,
                url: c.url,
                content: c.content,
                source_domain: c.source_domain,
                author: c.author,
                published_date: c.published_date.map(|d| d.to_rfc3339()),
                excerpt: c.excerpt,
                thumbnail_url: c.thumbnail_url,
                read_status: c.read_status,
                notes: c.notes,
                tags: c.tags,
                image_paths: c.image_paths,
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
