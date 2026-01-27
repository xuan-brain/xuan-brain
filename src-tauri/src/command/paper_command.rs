use sea_orm::{DatabaseConnection, EntityTrait, LoaderTrait, QueryOrder};
use serde::Serialize;
use tauri::State;
use tracing::{info, instrument};

use crate::database::entities::{papers, prelude::*};
use crate::sys::error::Result;

#[derive(Serialize)]
pub struct LabelDto {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct PaperDto {
    pub id: i64,
    pub title: String,
    pub publication_year: Option<i64>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub authors: Vec<String>,
    pub labels: Vec<LabelDto>,
}

#[derive(Serialize)]
pub struct PaperDetailDto {
    pub id: i64,
    pub title: String,
    pub abstract_text: Option<String>,
    pub doi: Option<String>,
    pub publication_year: Option<i64>,
    pub publication_date: Option<String>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub citation_count: Option<i64>,
    pub read_status: Option<String>,
    pub notes: Option<String>,
    pub authors: Vec<String>,
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, DatabaseConnection>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");
    let papers = Papers::find()
        .order_by_desc(papers::Column::Id)
        .all(db.inner())
        .await?;

    let authors = papers
        .load_many_to_many(Authors, PaperAuthors, db.inner())
        .await?;

    let labels = papers
        .load_many_to_many(Label, PaperLabels, db.inner())
        .await?;

    let dtos: Vec<PaperDto> = papers
        .into_iter()
        .zip(authors.into_iter())
        .zip(labels.into_iter())
        .map(|((paper, authors), labels)| PaperDto {
            id: paper.id,
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: authors.into_iter().map(|a| a.name).collect(),
            labels: labels
                .into_iter()
                .map(|l| LabelDto {
                    id: l.id,
                    name: l.name,
                    color: l.color,
                })
                .collect(),
        })
        .collect();

    info!("Fetched {} papers", dtos.len());
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_paper(
    id: i64,
    db: State<'_, DatabaseConnection>,
) -> Result<Option<PaperDetailDto>> {
    info!("Fetching details for paper id {}", id);
    let paper_with_authors = Papers::find_by_id(id)
        .find_with_related(Authors)
        .all(db.inner())
        .await?;

    if let Some((paper, authors)) = paper_with_authors.into_iter().next() {
        Ok(Some(PaperDetailDto {
            id: paper.id,
            title: paper.title,
            abstract_text: paper.r#abstract,
            doi: paper.doi,
            publication_year: paper.publication_year,
            publication_date: paper.publication_date,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            volume: paper.volume,
            issue: paper.issue,
            pages: paper.pages,
            url: paper.url,
            citation_count: paper.citation_count,
            read_status: paper.read_status,
            notes: paper.notes,
            authors: authors.into_iter().map(|a| a.name).collect(),
        }))
    } else {
        info!("Paper id {} not found", id);
        Ok(None)
    }
}
