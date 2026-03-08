//! Query operations for papers (read-only)

use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tauri::State;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::repository::{AuthorRepository, CategoryRepository, LabelRepository, PaperRepository};
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::parse_id;

/// DTO for paper count
#[derive(Serialize)]
pub struct PaperCountDto {
    pub total: i64,
    pub deleted: i64,
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_paper_count(db: State<'_, Arc<DatabaseConnection>>) -> Result<PaperCountDto> {
    info!("Getting paper count");

    let total = PaperRepository::count(&db).await?;
    let deleted = PaperRepository::count_deleted(&db).await?;

    info!("Paper count: {} total, {} deleted", total, deleted);
    Ok(PaperCountDto { total, deleted })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    let total_start = Instant::now();
    info!("[PERF] Starting get_all_papers (batch optimized)");

    // Step 1: Fetch all papers
    let step1_start = Instant::now();
    let papers = PaperRepository::find_all(&db).await?;
    let paper_count = papers.len();
    info!(
        "[PERF] Step 1 - find_all: {:?}ms, found {} papers",
        step1_start.elapsed().as_millis(),
        paper_count
    );

    if paper_count == 0 {
        return Ok(Vec::new());
    }

    // Collect all paper IDs for batch queries
    let paper_ids: Vec<i64> = papers.iter().map(|p| p.id).collect();

    // Step 2: Batch fetch attachments
    let step2_start = Instant::now();
    let attachments_map = PaperRepository::get_attachments_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 2 - batch attachments: {:?}ms",
        step2_start.elapsed().as_millis()
    );

    // Step 3: Batch fetch authors
    let step3_start = Instant::now();
    let authors_map = AuthorRepository::get_paper_authors_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 3 - batch authors: {:?}ms",
        step3_start.elapsed().as_millis()
    );

    // Step 4: Batch fetch labels
    let step4_start = Instant::now();
    let labels_map = LabelRepository::get_paper_labels_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 4 - batch labels: {:?}ms",
        step4_start.elapsed().as_millis()
    );

    // Step 5: Build result DTOs
    let step5_start = Instant::now();
    let result: Vec<PaperDto> = papers
        .into_iter()
        .map(|paper| {
            let attachments = attachments_map.get(&paper.id).cloned().unwrap_or_default();
            let authors = authors_map.get(&paper.id).cloned().unwrap_or_default();
            let labels = labels_map.get(&paper.id).cloned().unwrap_or_default();

            let attachment_dtos: Vec<AttachmentDto> = attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: a.id.to_string(),
                    paper_id: paper.id.to_string(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: Some(a.created_at.to_rfc3339()),
                })
                .collect();

            let author_names: Vec<String> = authors.iter().map(|a| a.full_name()).collect();

            let label_dtos: Vec<LabelDto> = labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.to_string(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();

            PaperDto {
                id: paper.id.to_string(),
                title: paper.title,
                publication_year: paper.publication_year,
                journal_name: paper.journal_name,
                conference_name: paper.conference_name,
                authors: author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
                publisher: paper.publisher,
                issn: paper.issn,
                language: paper.language,
            }
        })
        .collect();

    info!(
        "[PERF] Step 5 - build DTOs: {:?}ms",
        step5_start.elapsed().as_millis()
    );

    let total_time = total_start.elapsed().as_millis();
    info!(
        "[PERF] get_all_papers completed: total={}ms, papers={} (batch optimized, 4 queries instead of {})",
        total_time,
        result.len(),
        1 + paper_count * 3
    );

    Ok(result)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_deleted_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    let total_start = Instant::now();
    info!("[PERF] Starting get_deleted_papers (batch optimized)");

    let step1_start = Instant::now();
    let papers = PaperRepository::find_deleted(&db).await?;
    let paper_count = papers.len();
    info!(
        "[PERF] Step 1 - find_deleted: {:?}ms, found {} papers",
        step1_start.elapsed().as_millis(),
        paper_count
    );

    if paper_count == 0 {
        return Ok(Vec::new());
    }

    // Collect all paper IDs for batch queries
    let paper_ids: Vec<i64> = papers.iter().map(|p| p.id).collect();

    // Batch fetch all related data
    let batch_start = Instant::now();
    let attachments_map = PaperRepository::get_attachments_batch(&db, &paper_ids).await?;
    let attachments_time = batch_start.elapsed().as_millis();

    let authors_batch_start = Instant::now();
    let authors_map = AuthorRepository::get_paper_authors_batch(&db, &paper_ids).await?;
    let authors_time = authors_batch_start.elapsed().as_millis();

    let labels_batch_start = Instant::now();
    let labels_map = LabelRepository::get_paper_labels_batch(&db, &paper_ids).await?;
    let labels_time = labels_batch_start.elapsed().as_millis();

    info!(
        "[PERF] Batch queries: attachments={}ms, authors={}ms, labels={}ms",
        attachments_time, authors_time, labels_time
    );

    // Build result DTOs
    let result: Vec<PaperDto> = papers
        .into_iter()
        .map(|paper| {
            let attachments = attachments_map.get(&paper.id).cloned().unwrap_or_default();
            let authors = authors_map.get(&paper.id).cloned().unwrap_or_default();
            let labels = labels_map.get(&paper.id).cloned().unwrap_or_default();

            let attachment_dtos: Vec<AttachmentDto> = attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: a.id.to_string(),
                    paper_id: paper.id.to_string(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: Some(a.created_at.to_rfc3339()),
                })
                .collect();

            let author_names: Vec<String> = authors.iter().map(|a| a.full_name()).collect();

            let label_dtos: Vec<LabelDto> = labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.to_string(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();

            PaperDto {
                id: paper.id.to_string(),
                title: paper.title,
                publication_year: paper.publication_year,
                journal_name: paper.journal_name,
                conference_name: paper.conference_name,
                authors: author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
                publisher: paper.publisher,
                issn: paper.issn,
                language: paper.language,
            }
        })
        .collect();

    let total_time = total_start.elapsed().as_millis();
    info!(
        "[PERF] get_deleted_papers completed: total={}ms, papers={} (batch optimized)",
        total_time,
        result.len()
    );

    Ok(result)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_paper(
    id: String,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<Option<PaperDetailDto>> {
    info!("Fetching details for paper id {}", id);

    let id_num = parse_id(&id)
        .map_err(|_| AppError::validation("id", "Invalid id format"))?;

    let paper = PaperRepository::find_by_id(&db, id_num).await?;

    if let Some(paper) = paper {
        // Get authors
        let authors = AuthorRepository::get_paper_authors(&db, paper.id).await?;
        let author_names: Vec<String> = authors.iter().map(|a| a.full_name()).collect();

        // Get labels
        let labels = LabelRepository::get_paper_labels(&db, paper.id).await?;
        let label_dtos: Vec<LabelDto> = labels
            .iter()
            .map(|l| LabelDto {
                id: l.id.to_string(),
                name: l.name.clone(),
                color: l.color.clone(),
            })
            .collect();

        // Get category
        let category_id = PaperRepository::get_category_id(&db, paper.id).await?;
        let category_name = if let Some(cat_id) = category_id {
            CategoryRepository::find_by_id(&db, cat_id)
                .await?
                .map(|c| c.name)
        } else {
            None
        };

        // Get attachments
        let attachments = PaperRepository::get_attachments(&db, paper.id).await?;
        let attachment_dtos: Vec<AttachmentDto> = attachments
            .iter()
            .map(|a| AttachmentDto {
                id: a.id.to_string(),
                paper_id: paper.id.to_string(),
                file_name: a.file_name.clone(),
                file_type: a.file_type.clone(),
                created_at: Some(a.created_at.to_rfc3339()),
            })
            .collect();
        let attachment_count = attachment_dtos.len();

        Ok(Some(PaperDetailDto {
            id: paper.id.to_string(),
            title: paper.title,
            abstract_text: paper.abstract_text,
            doi: paper.doi,
            publication_year: paper.publication_year,
            publication_date: paper.publication_date,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            volume: paper.volume,
            issue: paper.issue,
            pages: paper.pages,
            url: paper.url,
            citation_count: Some(paper.citation_count),
            read_status: Some(paper.read_status),
            notes: paper.notes,
            authors: author_names,
            labels: label_dtos,
            category_id: category_id.map(|id| id.to_string()),
            category_name,
            attachments: attachment_dtos,
            attachment_count,
            created_at: Some(paper.created_at.to_rfc3339()),
            updated_at: Some(paper.updated_at.to_rfc3339()),
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        }))
    } else {
        info!("Paper id {} not found", id);
        Ok(None)
    }
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_papers_by_category(
    db: State<'_, Arc<DatabaseConnection>>,
    category_id: String,
) -> Result<Vec<PaperDto>> {
    let total_start = Instant::now();
    info!("[PERF] Starting get_papers_by_category for category: {} (batch optimized)", category_id);

    let category_id_num = parse_id(&category_id)
        .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;

    let step1_start = Instant::now();
    let papers = PaperRepository::find_by_category(&db, category_id_num).await?;
    let paper_count = papers.len();
    info!(
        "[PERF] Step 1 - find_by_category: {:?}ms, found {} papers",
        step1_start.elapsed().as_millis(),
        paper_count
    );

    if paper_count == 0 {
        return Ok(Vec::new());
    }

    // Collect all paper IDs for batch queries
    let paper_ids: Vec<i64> = papers.iter().map(|p| p.id).collect();

    // Batch fetch all related data
    let batch_start = Instant::now();
    let attachments_map = PaperRepository::get_attachments_batch(&db, &paper_ids).await?;
    let attachments_time = batch_start.elapsed().as_millis();

    let authors_batch_start = Instant::now();
    let authors_map = AuthorRepository::get_paper_authors_batch(&db, &paper_ids).await?;
    let authors_time = authors_batch_start.elapsed().as_millis();

    let labels_batch_start = Instant::now();
    let labels_map = LabelRepository::get_paper_labels_batch(&db, &paper_ids).await?;
    let labels_time = labels_batch_start.elapsed().as_millis();

    info!(
        "[PERF] Batch queries: attachments={}ms, authors={}ms, labels={}ms",
        attachments_time, authors_time, labels_time
    );

    // Build result DTOs
    let result: Vec<PaperDto> = papers
        .into_iter()
        .map(|paper| {
            let attachments = attachments_map.get(&paper.id).cloned().unwrap_or_default();
            let authors = authors_map.get(&paper.id).cloned().unwrap_or_default();
            let labels = labels_map.get(&paper.id).cloned().unwrap_or_default();

            let attachment_dtos: Vec<AttachmentDto> = attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: a.id.to_string(),
                    paper_id: paper.id.to_string(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: Some(a.created_at.to_rfc3339()),
                })
                .collect();

            let author_names: Vec<String> = authors.iter().map(|a| a.full_name()).collect();

            let label_dtos: Vec<LabelDto> = labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.to_string(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();

            PaperDto {
                id: paper.id.to_string(),
                title: paper.title,
                publication_year: paper.publication_year,
                journal_name: paper.journal_name,
                conference_name: paper.conference_name,
                authors: author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
                publisher: paper.publisher,
                issn: paper.issn,
                language: paper.language,
            }
        })
        .collect();

    let total_time = total_start.elapsed().as_millis();
    info!(
        "[PERF] get_papers_by_category completed: total={}ms, papers={} (batch optimized)",
        total_time,
        result.len()
    );

    Ok(result)
}
