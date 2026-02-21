//! Query operations for papers (read-only)

use std::sync::Arc;
use tauri::State;
use tracing::{info, instrument};

use crate::repository::{AttachmentRepository, AuthorRepository, CategoryRepository, LabelRepository, PaperRepository};
use crate::surreal::connection::SurrealClient;
use crate::sys::error::Result;

use super::dtos::*;
use super::utils::record_id_to_string;

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");
    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);
    let label_repo = LabelRepository::new(&db);
    let attachment_repo = AttachmentRepository::new(&db);

    let papers = paper_repo.find_all().await?;

    let mut result = Vec::new();
    for paper in papers {
        let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

        let authors = author_repo.get_paper_authors(&paper_id).await.unwrap_or_default();
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

        let labels = label_repo.get_paper_labels(&paper_id).await.unwrap_or_default();
        let label_dtos: Vec<LabelDto> = labels.iter().map(|l| LabelDto {
            id: l.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            name: l.name.clone(),
            color: l.color.clone(),
        }).collect();

        let attachments = attachment_repo.find_by_paper(&paper_id).await.unwrap_or_default();
        let attachment_dtos: Vec<AttachmentDto> = attachments.iter().map(|a| AttachmentDto {
            id: a.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            paper_id: paper_id.clone(),
            file_name: a.file_name.clone(),
            file_type: a.file_type.clone(),
            created_at: None,
        }).collect();

        result.push(PaperDto {
            id: paper_id,
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
pub async fn get_deleted_papers(db: State<'_, Arc<SurrealClient>>) -> Result<Vec<PaperDto>> {
    info!("Fetching deleted papers");
    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);
    let label_repo = LabelRepository::new(&db);
    let attachment_repo = AttachmentRepository::new(&db);

    let papers = paper_repo.find_deleted().await?;

    let mut result = Vec::new();
    for paper in papers {
        let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

        let authors = author_repo.get_paper_authors(&paper_id).await.unwrap_or_default();
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

        let labels = label_repo.get_paper_labels(&paper_id).await.unwrap_or_default();
        let label_dtos: Vec<LabelDto> = labels.iter().map(|l| LabelDto {
            id: l.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            name: l.name.clone(),
            color: l.color.clone(),
        }).collect();

        let attachments = attachment_repo.find_by_paper(&paper_id).await.unwrap_or_default();
        let attachment_dtos: Vec<AttachmentDto> = attachments.iter().map(|a| AttachmentDto {
            id: a.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            paper_id: paper_id.clone(),
            file_name: a.file_name.clone(),
            file_type: a.file_type.clone(),
            created_at: None,
        }).collect();

        result.push(PaperDto {
            id: paper_id,
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
    db: State<'_, Arc<SurrealClient>>,
) -> Result<Option<PaperDetailDto>> {
    info!("Fetching details for paper id {}", id);
    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);
    let label_repo = LabelRepository::new(&db);
    let category_repo = CategoryRepository::new(&db);

    let paper = paper_repo.find_by_id(&id).await?;
    if let Some(paper) = paper {
        let authors = author_repo.get_paper_authors(&id).await.unwrap_or_default();
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

        let labels = label_repo.get_paper_labels(&id).await.unwrap_or_default();
        let label_dtos: Vec<LabelDto> = labels.iter().map(|l| LabelDto {
            id: l.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            name: l.name.clone(),
            color: l.color.clone(),
        }).collect();

        let category_id = paper_repo.get_category_id(&id).await?;
        let category_name = if let Some(cat_id) = &category_id {
            category_repo.find_by_id(cat_id).await?.map(|c| c.name)
        } else {
            None
        };

        Ok(Some(PaperDetailDto {
            id: paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
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
    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);
    let label_repo = LabelRepository::new(&db);
    let attachment_repo = AttachmentRepository::new(&db);

    let papers = paper_repo.find_by_category(&category_id).await?;

    let mut result = Vec::new();
    for paper in papers {
        let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

        let authors = author_repo.get_paper_authors(&paper_id).await.unwrap_or_default();
        let author_names: Vec<String> = authors.iter().map(|a| a.name.clone()).collect();

        let labels = label_repo.get_paper_labels(&paper_id).await.unwrap_or_default();
        let label_dtos: Vec<LabelDto> = labels.iter().map(|l| LabelDto {
            id: l.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            name: l.name.clone(),
            color: l.color.clone(),
        }).collect();

        let attachments = attachment_repo.find_by_paper(&paper_id).await.unwrap_or_default();
        let attachment_dtos: Vec<AttachmentDto> = attachments.iter().map(|a| AttachmentDto {
            id: a.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default(),
            paper_id: paper_id.clone(),
            file_name: a.file_name.clone(),
            file_type: a.file_type.clone(),
            created_at: None,
        }).collect();

        result.push(PaperDto {
            id: paper_id,
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
