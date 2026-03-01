//! Query operations for papers (read-only)

use std::sync::Arc;

use tauri::State;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::repository::{AuthorRepository, CategoryRepository, LabelRepository, PaperRepository};
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::parse_id;

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");

    let papers = PaperRepository::find_all(&db).await?;
    let mut result = Vec::new();

    for paper in papers {
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

        // Get authors
        let authors = AuthorRepository::get_paper_authors(&db, paper.id).await?;
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

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

        result.push(PaperDto {
            id: paper.id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: label_dtos,
            attachment_count: attachment_dtos.len(),
            attachments: attachment_dtos,
        });
    }

    info!("Fetched {} papers", result.len());
    Ok(result)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_deleted_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    info!("Fetching deleted papers");

    let papers = PaperRepository::find_deleted(&db).await?;
    let mut result = Vec::new();

    for paper in papers {
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

        // Get authors
        let authors = AuthorRepository::get_paper_authors(&db, paper.id).await?;
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

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

        result.push(PaperDto {
            id: paper.id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: label_dtos,
            attachment_count: attachment_dtos.len(),
            attachments: attachment_dtos,
        });
    }

    info!("Fetched {} deleted papers", result.len());
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
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

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
    info!("Fetching papers for category id: {}", category_id);

    let category_id_num = parse_id(&category_id)
        .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;

    let papers = PaperRepository::find_by_category(&db, category_id_num).await?;
    let mut result = Vec::new();

    for paper in papers {
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

        // Get authors
        let authors = AuthorRepository::get_paper_authors(&db, paper.id).await?;
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

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

        result.push(PaperDto {
            id: paper.id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: label_dtos,
            attachment_count: attachment_dtos.len(),
            attachments: attachment_dtos,
        });
    }

    info!("Fetched {} papers for category {}", result.len(), category_id);
    Ok(result)
}
