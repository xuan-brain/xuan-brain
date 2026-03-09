//! Query operations for papers (read-only)

use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;
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

/// DTO for paginated papers response
#[derive(Serialize)]
pub struct PaginatedPapersDto {
    pub papers: Vec<PaperDto>,
    pub total: i64,
    pub offset: u64,
    pub limit: u64,
    pub has_more: bool,
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

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_papers_paginated(
    db: State<'_, Arc<DatabaseConnection>>,
    offset: u64,
    limit: u64,
) -> Result<PaginatedPapersDto> {
    let total_start = Instant::now();
    info!(
        "[PERF] Starting get_papers_paginated (offset={}, limit={})",
        offset, limit
    );

    // Step 1: Get total count
    let total = PaperRepository::count(&db).await?;

    // Step 2: Fetch paginated papers
    let step2_start = Instant::now();
    let papers = PaperRepository::find_all_paginated(&db, offset, limit).await?;
    let paper_count = papers.len();
    info!(
        "[PERF] Step 2 - find_paginated: {:?}ms, found {} papers",
        step2_start.elapsed().as_millis(),
        paper_count
    );

    if paper_count == 0 {
        return Ok(PaginatedPapersDto {
            papers: Vec::new(),
            total,
            offset,
            limit,
            has_more: false,
        });
    }

    // Collect paper IDs for batch queries
    let paper_ids: Vec<i64> = papers.iter().map(|p| p.id).collect();

    // Step 3: Batch fetch attachments
    let step3_start = Instant::now();
    let attachments_map = PaperRepository::get_attachments_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 3 - batch attachments: {:?}ms",
        step3_start.elapsed().as_millis()
    );

    // Step 4: Batch fetch authors
    let step4_start = Instant::now();
    let authors_map = AuthorRepository::get_paper_authors_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 4 - batch authors: {:?}ms",
        step4_start.elapsed().as_millis()
    );

    // Step 5: Batch fetch labels
    let step5_start = Instant::now();
    let labels_map = LabelRepository::get_paper_labels_batch(&db, &paper_ids).await?;
    info!(
        "[PERF] Step 5 - batch labels: {:?}ms",
        step5_start.elapsed().as_millis()
    );

    // Step 6: Build result DTOs
    let step6_start = Instant::now();
    let paper_dtos: Vec<PaperDto> = papers
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
        "[PERF] Step 6 - build DTOs: {:?}ms",
        step6_start.elapsed().as_millis()
    );

    let has_more = (offset + paper_count as u64) < total as u64;
    let total_time = total_start.elapsed().as_millis();

    info!(
        "[PERF] get_papers_paginated completed: total={}ms, papers={}, has_more={}",
        total_time,
        paper_dtos.len(),
        has_more
    );

    Ok(PaginatedPapersDto {
        papers: paper_dtos,
        total,
        offset,
        limit,
        has_more,
    })
}

/// Stream all papers - returns first batch synchronously, rest via Channel
/// This ensures immediate display of first batch without waiting for async events
#[tauri::command]
#[instrument(skip(db, channel))]
pub async fn stream_all_papers(
    db: State<'_, Arc<DatabaseConnection>>,
    channel: Channel<PaperBatchDto>,
) -> Result<StreamInitDto> {
    // Two-phase loading: first batch is smaller for faster display
    const FIRST_BATCH_SIZE: usize = 30; // Small first batch returned synchronously
    const SUBSEQUENT_BATCH_SIZE: usize = 100; // Larger batches via Channel

    let total_start = Instant::now();
    info!("[PERF] === DETAILED PROFILING START ===");

    // OPTIMIZATION: Run COUNT and first batch query IN PARALLEL
    // This saves ~87ms by not waiting for COUNT before querying papers
    let t1 = Instant::now();
    let (count_result, papers_result) = tokio::join!(
        PaperRepository::count(&db),
        PaperRepository::find_all_paginated(&db, 0, FIRST_BATCH_SIZE as u64)
    );
    let t1_elapsed = t1.elapsed();
    info!("[PERF] Step 1 - parallel count + query: {}ms", t1_elapsed.as_millis());

    let total = count_result? as usize;
    let first_papers = papers_result?;

    if total == 0 {
        info!("[PERF] No papers to stream");
        return Ok(StreamInitDto {
            first_batch: Vec::new(),
            total: 0,
            first_batch_count: 0,
            has_more: false,
        });
    }

    let first_paper_ids: Vec<i64> = first_papers.iter().map(|p| p.id).collect();

    // Step 2: Batch fetch authors only (no attachments or labels - use paper.attachment_count directly)
    let t2 = Instant::now();
    let authors_map = AuthorRepository::get_paper_authors_batch(&db, &first_paper_ids).await?;
    let t2_elapsed = t2.elapsed();
    info!("[PERF] Step 2 - batch authors ONLY (using paper.attachment_count): {}ms", t2_elapsed.as_millis());

    // Step 3: Build lightweight DTOs using paper.attachment_count directly
    let t3 = Instant::now();
    let first_batch: Vec<PaperListDto> = first_papers
        .into_iter()
        .map(|paper| {
            let authors = authors_map
                .get(&paper.id)
                .cloned()
                .unwrap_or_default();

            let author_names: Vec<String> = authors.iter().map(|a| a.full_name()).collect();

            // Use attachment_count from paper model directly (no attachment query needed)
            PaperListDto {
                id: paper.id.to_string(),
                title: paper.title,
                publication_year: paper.publication_year,
                journal_name: paper.journal_name,
                conference_name: paper.conference_name,
                authors: author_names,
                attachment_count: paper.attachment_count as usize,
            }
        })
        .collect();

    let t3_elapsed = t3.elapsed();
    info!("[PERF] Step 3 - build lightweight DTOs: {}ms", t3_elapsed.as_millis());

    let first_batch_count = first_batch.len();
    let has_more = first_batch_count < total;

    let first_batch_total = total_start.elapsed();
    info!(
        "[PERF] === FIRST BATCH TOTAL: {}ms (count={}, has_more={}) ===",
        first_batch_total.as_millis(),
        first_batch_count,
        has_more
    );

    // Step 4: Stream remaining batches via Channel (in background)
    if has_more {
        let mut offset = FIRST_BATCH_SIZE as u64;
        let mut batch_index = 1; // Start from 1 since first batch is index 0
        let mut loaded_count = first_batch_count;

        loop {
            let batch_start = Instant::now();

            let papers =
                PaperRepository::find_all_paginated(&db, offset, SUBSEQUENT_BATCH_SIZE as u64)
                    .await?;

            if papers.is_empty() {
                break;
            }

            let paper_ids: Vec<i64> = papers.iter().map(|p| p.id).collect();

            // Batch fetch authors only (no attachments or labels)
            let authors_map = AuthorRepository::get_paper_authors_batch(&db, &paper_ids).await?;

            // Build lightweight DTOs using paper.attachment_count directly
            let paper_dtos: Vec<PaperListDto> = papers
                .into_iter()
                .map(|paper| {
                    let authors = authors_map.get(&paper.id).cloned().unwrap_or_default();

                    let author_names: Vec<String> =
                        authors.iter().map(|a| a.full_name()).collect();

                    // Use attachment_count from paper model directly
                    PaperListDto {
                        id: paper.id.to_string(),
                        title: paper.title,
                        publication_year: paper.publication_year,
                        journal_name: paper.journal_name,
                        conference_name: paper.conference_name,
                        authors: author_names,
                        attachment_count: paper.attachment_count as usize,
                    }
                })
                .collect();

            loaded_count += paper_dtos.len();
            let is_last = loaded_count >= total;

            let batch_time = batch_start.elapsed().as_millis();
            info!(
                "[PERF] Channel batch {}: sent {} papers in {}ms (total: {}/{})",
                batch_index,
                paper_dtos.len(),
                batch_time,
                loaded_count,
                total
            );

            // Send via Channel
            channel.send(PaperBatchDto {
                papers: paper_dtos,
                batch_index,
                is_last,
                loaded_count,
                total,
            }).map_err(|e| AppError::generic(format!("Failed to send channel batch: {}", e)))?;

            if is_last {
                break;
            }

            offset += SUBSEQUENT_BATCH_SIZE as u64;
            batch_index += 1;
        }
    }

    let total_time = total_start.elapsed().as_millis();
    info!(
        "[PERF] stream_all_papers completed: {} papers total, first_batch={} (sync), rest_via_channel={}, total={}ms",
        total,
        first_batch_count,
        has_more,
        total_time
    );

    // Return first batch synchronously
    Ok(StreamInitDto {
        first_batch,
        total,
        first_batch_count,
        has_more,
    })
}
