use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::Serialize;
use tauri::State;
use tracing::{info, instrument};

use crate::database::entities::{authors, paper_authors, papers, prelude::*};
use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::sys::error::{AppError, Result};

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

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_doi(
    doi: String,
    db: State<'_, DatabaseConnection>,
) -> Result<PaperDto> {
    info!("Importing paper with DOI: {}", doi);

    // Fetch metadata from DOI
    let metadata = fetch_doi_metadata(&doi).await.map_err(|e| match e {
        DoiError::InvalidDoi(doi) => AppError::validation("doi", format!("Invalid DOI: {}", doi)),
        DoiError::NotFound => AppError::not_found("DOI", doi),
        DoiError::ParseError(msg) => {
            AppError::validation("metadata", format!("Failed to parse DOI metadata: {}", msg))
        }
        DoiError::RequestError(e) => {
            AppError::network_error(&doi, format!("Failed to fetch DOI: {}", e))
        }
    })?;

    // Check if paper already exists
    if let Some(existing) = Papers::find()
        .filter(papers::Column::Doi.eq(&metadata.doi))
        .one(db.inner())
        .await?
    {
        info!(
            "Paper with DOI {} already exists, id: {}",
            metadata.doi, existing.id
        );
        return Err(AppError::validation(
            "doi",
            format!("Paper with DOI {} already exists", metadata.doi),
        ));
    }

    // Parse publication year
    let publication_year = metadata
        .publication_year
        .and_then(|y| y.parse::<i64>().ok());

    // Create paper
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(Some(metadata.doi.clone())),
        publication_year: Set(publication_year),
        journal_name: Set(metadata.journal_name.clone()),
        url: Set(metadata.url.clone()),
        r#abstract: Set(metadata.abstract_text.clone()),
        ..Default::default()
    }
    .insert(db.inner())
    .await?;

    // Add authors
    for author_name in &metadata.authors {
        // Find or create author
        let author = if let Some(existing_author) = Authors::find()
            .filter(authors::Column::Name.eq(author_name))
            .one(db.inner())
            .await?
        {
            existing_author
        } else {
            authors::ActiveModel {
                name: Set(author_name.clone()),
                ..Default::default()
            }
            .insert(db.inner())
            .await?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db.inner())
        .await?;
    }

    info!(
        "Successfully imported paper: {} (id: {}, doi: {})",
        metadata.title, paper.id, metadata.doi
    );

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_arxiv_id(
    arxiv_id: String,
    db: State<'_, DatabaseConnection>,
) -> Result<PaperDto> {
    info!("Importing paper with arXiv ID: {}", arxiv_id);

    // Fetch metadata from arXiv
    let metadata = fetch_arxiv_metadata(&arxiv_id).await.map_err(|e| match e {
        ArxivError::InvalidArxivId(id) => {
            AppError::validation("arxiv_id", format!("Invalid arXiv ID: {}", id))
        }
        ArxivError::NotFound => AppError::not_found("arXiv ID", arxiv_id),
        ArxivError::ParseError(msg) => AppError::validation(
            "metadata",
            format!("Failed to parse arXiv metadata: {}", msg),
        ),
        ArxivError::RequestError(e) => {
            AppError::network_error(&arxiv_id, format!("Failed to fetch arXiv: {}", e))
        }
    })?;

    // Check if paper already exists by DOI or URL
    if let Some(doi) = &metadata.doi {
        if let Some(existing) = Papers::find()
            .filter(papers::Column::Doi.eq(doi))
            .one(db.inner())
            .await?
        {
            info!("Paper with DOI {} already exists, id: {}", doi, existing.id);
            return Err(AppError::validation(
                "doi",
                format!("Paper with DOI {} already exists", doi),
            ));
        }
    }

    // Extract publication year from published date
    let publication_year = metadata
        .published
        .split('-')
        .next()
        .and_then(|y| y.parse::<i64>().ok());

    // Create paper
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(metadata.doi.clone()),
        publication_year: Set(publication_year),
        url: Set(Some(metadata.pdf_url.clone())),
        r#abstract: Set(Some(metadata.summary.clone())),
        journal_name: Set(metadata.journal_ref.clone()),
        ..Default::default()
    }
    .insert(db.inner())
    .await?;

    // Add authors
    for author_name in &metadata.authors {
        // Find or create author
        let author = if let Some(existing_author) = Authors::find()
            .filter(authors::Column::Name.eq(author_name))
            .one(db.inner())
            .await?
        {
            existing_author
        } else {
            authors::ActiveModel {
                name: Set(author_name.clone()),
                ..Default::default()
            }
            .insert(db.inner())
            .await?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db.inner())
        .await?;
    }

    info!(
        "Successfully imported arXiv paper: {} (id: {}, arxiv_id: {})",
        metadata.title, paper.id, metadata.arxiv_id
    );

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
    })
}
