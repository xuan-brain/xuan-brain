//! Utility functions for clip commands

use std::fs;
use std::path::PathBuf;

use regex::Regex;
use sha1::{Digest, Sha1};
use tracing::{info, warn};

use crate::sys::error::{AppError, Result};

/// Convert RecordId to string
pub fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    use surrealdb_types::RecordIdKey;
    format!(
        "{}:{}",
        id.table,
        match &id.key {
            RecordIdKey::String(s) => s.clone(),
            RecordIdKey::Number(n) => n.to_string(),
            RecordIdKey::Uuid(u) => u.to_string(),
            _ => "unknown".to_string(),
        }
    )
}

/// Extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    if let Some(parsed) = url.split('?').next() {
        if let Some(filename) = parsed.split('/').next_back() {
            if !filename.is_empty() {
                if filename.contains('.') {
                    return filename.to_string();
                }
                // Generate hash-based filename for URLs without extension
                let mut hasher = Sha1::new();
                hasher.update(url.as_bytes());
                return format!("{:x}.jpg", hasher.finalize());
            }
        }
    }
    // Fallback: generate hash-based filename
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    format!("{:x}.jpg", hasher.finalize())
}

/// Download image and save to local storage
async fn download_image(url: &str, clip_id: &str, files_dir: &str) -> Result<String> {
    let filename = extract_filename_from_url(url);
    let clip_dir = PathBuf::from(files_dir)
        .join("clips")
        .join(clip_id)
        .join("images");

    fs::create_dir_all(&clip_dir).map_err(|e| {
        AppError::file_system(
            clip_dir.display().to_string(),
            format!("Failed to create images directory: {}", e),
        )
    })?;

    let response = reqwest::get(url).await.map_err(|e| {
        AppError::generic(format!("Failed to download image from {}: {}", url, e))
    })?;

    if !response.status().is_success() {
        return Err(AppError::generic(format!(
            "Failed to download image from {}: HTTP {}",
            url,
            response.status()
        )));
    }

    let bytes = response.bytes().await.map_err(|e| {
        AppError::generic(format!("Failed to read image bytes from {}: {}", url, e))
    })?;

    let local_path = clip_dir.join(&filename);
    fs::write(&local_path, bytes).map_err(|e| {
        AppError::file_system(
            local_path.display().to_string(),
            format!("Failed to write image: {}", e),
        )
    })?;

    info!("Downloaded image from {} to {:?}", url, local_path);
    Ok(format!("/clips/images/{}/images/{}", clip_id, filename))
}

/// Process markdown content to download and replace image URLs
pub async fn process_markdown_images(
    content: String,
    clip_id: &str,
    files_dir: &str,
) -> Result<(String, Vec<String>)> {
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
