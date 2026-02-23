//! Query operations for papers (read-only)

use std::sync::Arc;
use surrealdb_types::SurrealValue;
use tauri::State;
use tracing::{info, instrument};

use crate::repository::{CategoryRepository, PaperRepository};
use crate::surreal::connection::SurrealClient;
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::record_id_to_string;

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");
    
    // Single query to fetch all papers with their related authors and labels
    // This eliminates the N+1 query problem
    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct PaperRow {
        id: Option<surrealdb_types::RecordId>,
        title: String,
        publication_year: Option<i32>,
        journal_name: Option<String>,
        conference_name: Option<String>,
        attachments: Vec<crate::surreal::models::paper::AttachmentEmbedded>,
        #[serde(default)]
        author_names: Vec<String>,
        #[serde(default)]
        labels: Vec<LabelRow>,
    }

    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct LabelRow {
        id: Option<surrealdb_types::RecordId>,
        name: String,
        color: String,
    }

    let result: Vec<PaperRow> = db
        .query(
            r#"
            SELECT
                id,
                title,
                publication_year,
                journal_name,
                conference_name,
                attachments,
                (SELECT VALUE name FROM author WHERE id IN 
                    (SELECT VALUE `out` FROM paper_author WHERE `in` = $parent.id)
                    ORDER BY (SELECT VALUE author_order FROM paper_author 
                             WHERE `in` = $parent.id AND `out` = author.id)[0]
                ) AS author_names,
                (SELECT id, name, color FROM label WHERE id IN 
                    (SELECT VALUE `out` FROM paper_label WHERE `in` = $parent.id)
                ) AS labels
            FROM paper
            WHERE deleted_at IS NONE
            ORDER BY created_at DESC
            "#,
        )
        .await
        .map_err(|e| AppError::generic(format!("Failed to fetch papers: {}", e)))?
        .take(0)
        .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

    let papers: Vec<PaperDto> = result
        .into_iter()
        .map(|p| {
            let paper_id = p.id.as_ref().map(record_id_to_string).unwrap_or_default();
            let label_dtos: Vec<LabelDto> = p.labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.as_ref().map(record_id_to_string).unwrap_or_default(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();
            let attachment_dtos: Vec<AttachmentDto> = p.attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: paper_id.clone(),
                    paper_id: paper_id.clone(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: None,
                })
                .collect();

            PaperDto {
                id: paper_id,
                title: p.title,
                publication_year: p.publication_year,
                journal_name: p.journal_name,
                conference_name: p.conference_name,
                authors: p.author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
            }
        })
        .collect();

    info!("Fetched {} papers", papers.len());
    Ok(papers)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_deleted_papers(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<PaperDto>> {
    info!("Fetching deleted papers");
    
    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct PaperRow {
        id: Option<surrealdb_types::RecordId>,
        title: String,
        publication_year: Option<i32>,
        journal_name: Option<String>,
        conference_name: Option<String>,
        attachments: Vec<crate::surreal::models::paper::AttachmentEmbedded>,
        #[serde(default)]
        author_names: Vec<String>,
        #[serde(default)]
        labels: Vec<LabelRow>,
    }

    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct LabelRow {
        id: Option<surrealdb_types::RecordId>,
        name: String,
        color: String,
    }

    let result: Vec<PaperRow> = db
        .query(
            r#"
            SELECT
                id,
                title,
                publication_year,
                journal_name,
                conference_name,
                attachments,
                (SELECT VALUE name FROM author WHERE id IN 
                    (SELECT VALUE `out` FROM paper_author WHERE `in` = $parent.id)
                    ORDER BY (SELECT VALUE author_order FROM paper_author 
                             WHERE `in` = $parent.id AND `out` = author.id)[0]
                ) AS author_names,
                (SELECT id, name, color FROM label WHERE id IN 
                    (SELECT VALUE `out` FROM paper_label WHERE `in` = $parent.id)
                ) AS labels
            FROM paper
            WHERE deleted_at IS NOT NONE
            ORDER BY deleted_at DESC
            "#,
        )
        .await
        .map_err(|e| AppError::generic(format!("Failed to fetch deleted papers: {}", e)))?
        .take(0)
        .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

    let papers: Vec<PaperDto> = result
        .into_iter()
        .map(|p| {
            let paper_id = p.id.as_ref().map(record_id_to_string).unwrap_or_default();
            let label_dtos: Vec<LabelDto> = p.labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.as_ref().map(record_id_to_string).unwrap_or_default(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();
            let attachment_dtos: Vec<AttachmentDto> = p.attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: paper_id.clone(),
                    paper_id: paper_id.clone(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: None,
                })
                .collect();

            PaperDto {
                id: paper_id,
                title: p.title,
                publication_year: p.publication_year,
                journal_name: p.journal_name,
                conference_name: p.conference_name,
                authors: p.author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
            }
        })
        .collect();

    info!("Fetched {} deleted papers", papers.len());
    Ok(papers)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_paper(
    id: String,
    db: State<'_, Arc<SurrealClient>>,
) -> Result<Option<PaperDetailDto>> {
    info!("Fetching details for paper id {}", id);
    let paper_repo = PaperRepository::new(&db);
    let category_repo = CategoryRepository::new(&db);

    let paper = paper_repo.find_by_id(&id).await?;
    if let Some(paper) = paper {
        // Fetch authors in a single subquery
        let author_names: Vec<String> = db
            .query(
                r#"
                SELECT VALUE name FROM author 
                WHERE id IN (SELECT VALUE `out` FROM paper_author WHERE `in` = type::record($paper))
                ORDER BY (SELECT VALUE author_order FROM paper_author 
                         WHERE `in` = type::record($paper) AND `out` = author.id)[0]
                "#,
            )
            .bind(("paper", id.clone()))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get authors: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        // Fetch labels in a single subquery
        #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
        struct LabelRow {
            id: Option<surrealdb_types::RecordId>,
            name: String,
            color: String,
        }
        let labels: Vec<LabelRow> = db
            .query(
                r#"
                SELECT id, name, color FROM label 
                WHERE id IN (SELECT VALUE `out` FROM paper_label WHERE `in` = type::record($paper))
                "#,
            )
            .bind(("paper", id.clone()))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get labels: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        let label_dtos: Vec<LabelDto> = labels
            .iter()
            .map(|l| LabelDto {
                id: l.id.as_ref().map(record_id_to_string).unwrap_or_default(),
                name: l.name.clone(),
                color: l.color.clone(),
            })
            .collect();

        let category_id = paper_repo.get_category_id(&id).await?;
        let category_name = if let Some(cat_id) = &category_id {
            category_repo.find_by_id(cat_id).await?.map(|c| c.name)
        } else {
            None
        };

        Ok(Some(PaperDetailDto {
            id: paper
                .id
                .as_ref()
                .map(record_id_to_string)
                .unwrap_or_default(),
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
            category_id,
            category_name,
        }))
    } else {
        info!("Paper id {} not found", id);
        Ok(None)
    }
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_papers_by_category(
    db: State<'_, Arc<SurrealClient>>,
    category_id: String,
) -> Result<Vec<PaperDto>> {
    info!("Fetching papers for category id: {}", category_id);

    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct PaperRow {
        id: Option<surrealdb_types::RecordId>,
        title: String,
        publication_year: Option<i32>,
        journal_name: Option<String>,
        conference_name: Option<String>,
        attachments: Vec<crate::surreal::models::paper::AttachmentEmbedded>,
        #[serde(default)]
        author_names: Vec<String>,
        #[serde(default)]
        labels: Vec<LabelRow>,
    }

    #[derive(Debug, serde::Deserialize, surrealdb_types::SurrealValue)]
    struct LabelRow {
        id: Option<surrealdb_types::RecordId>,
        name: String,
        color: String,
    }

    let result: Vec<PaperRow> = db
        .query(
            r#"
            SELECT
                id,
                title,
                publication_year,
                journal_name,
                conference_name,
                attachments,
                (SELECT VALUE name FROM author WHERE id IN 
                    (SELECT VALUE `out` FROM paper_author WHERE `in` = $parent.id)
                    ORDER BY (SELECT VALUE author_order FROM paper_author 
                             WHERE `in` = $parent.id AND `out` = author.id)[0]
                ) AS author_names,
                (SELECT id, name, color FROM label WHERE id IN 
                    (SELECT VALUE `out` FROM paper_label WHERE `in` = $parent.id)
                ) AS labels
            FROM paper
            WHERE deleted_at IS NONE
            AND id IN (SELECT VALUE `in` FROM paper_category WHERE `out` = type::record($category))
            ORDER BY created_at DESC
            "#,
        )
        .bind(("category", category_id.clone()))
        .await
        .map_err(|e| AppError::generic(format!("Failed to fetch papers: {}", e)))?
        .take(0)
        .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

    let papers: Vec<PaperDto> = result
        .into_iter()
        .map(|p| {
            let paper_id = p.id.as_ref().map(record_id_to_string).unwrap_or_default();
            let label_dtos: Vec<LabelDto> = p.labels
                .iter()
                .map(|l| LabelDto {
                    id: l.id.as_ref().map(record_id_to_string).unwrap_or_default(),
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect();
            let attachment_dtos: Vec<AttachmentDto> = p.attachments
                .iter()
                .map(|a| AttachmentDto {
                    id: paper_id.clone(),
                    paper_id: paper_id.clone(),
                    file_name: a.file_name.clone(),
                    file_type: a.file_type.clone(),
                    created_at: None,
                })
                .collect();

            PaperDto {
                id: paper_id,
                title: p.title,
                publication_year: p.publication_year,
                journal_name: p.journal_name,
                conference_name: p.conference_name,
                authors: p.author_names,
                labels: label_dtos,
                attachment_count: attachment_dtos.len(),
                attachments: attachment_dtos,
            }
        })
        .collect();

    info!("Fetched {} papers for category {}", papers.len(), category_id);
    Ok(papers)
}
