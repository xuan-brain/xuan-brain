use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, ModelTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tracing::{info, instrument};

use crate::database::entities::{
    attachments, authors, paper_authors, paper_category, paper_labels, papers, prelude::*,
};
use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::papers::importer::grobid::process_header_document;
use crate::papers::importer::pubmed::{fetch_pubmed_metadata, PubmedError};
use crate::sys::config::AppConfig;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

#[derive(Serialize)]
pub struct LabelDto {
    pub id: i64,
    pub name: String,
    pub color: String,
}

#[derive(Serialize)]
pub struct AttachmentDto {
    pub id: i64,
    pub paper_id: i64,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Serialize)]
pub struct PdfAttachmentInfo {
    pub file_path: String,
    pub file_name: String,
    pub paper_id: i64,
    pub paper_title: String,
    pub base64_content: Option<String>, // Will be populated if requested
}

#[derive(Serialize)]
pub struct PdfAttachmentBase64 {
    pub file_name: String,
    pub base64_content: String,
    pub paper_title: String,
}

#[derive(Serialize)]
pub struct PdfBlobResponse {
    pub file_name: String,
    pub paper_title: String,
    pub paper_id: i64,
    pub base64_data: String,
    pub size_bytes: usize,
}

#[derive(Deserialize)]
pub struct PdfBlobRequest {
    pub paper_id: i64,
}

#[derive(Deserialize)]
pub struct PdfBlobSaveRequest {
    pub paper_id: i64,
    pub base64_data: String,
}

#[derive(Serialize)]
pub struct PdfSaveResponse {
    pub success: bool,
    pub file_path: String,
    pub size_bytes: usize,
    pub message: String,
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
    pub attachment_count: usize,
    pub attachments: Vec<AttachmentDto>,
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
    pub labels: Vec<LabelDto>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct UpdatePaperDto {
    pub id: i64,
    pub title: String,
    pub publication_year: Option<i64>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pages: Option<String>,
    pub url: Option<String>,
    pub doi: Option<String>,
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
    pub read_status: Option<String>,
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_all_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");
    let db = db.inner().as_ref();
    let papers = Papers::find()
        .filter(papers::Column::DeletedAt.is_null())
        .order_by_desc(papers::Column::Id)
        .all(db)
        .await?;

    let authors = papers.load_many_to_many(Authors, PaperAuthors, db).await?;

    let labels = papers.load_many_to_many(Label, PaperLabels, db).await?;

    let attachments = papers.load_many(Attachments, db).await?;

    let dtos: Vec<PaperDto> = papers
        .into_iter()
        .zip(authors.into_iter())
        .zip(labels.into_iter())
        .zip(attachments.into_iter())
        .map(|(((paper, authors), labels), attachments)| PaperDto {
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
            attachment_count: attachments.len(),
            attachments: attachments
                .into_iter()
                .map(|a| AttachmentDto {
                    id: a.id,
                    paper_id: a.paper_id,
                    file_name: a.file_name,
                    file_type: a.file_type,
                    created_at: a.created_at.map(|d| d.to_string()),
                })
                .collect(),
        })
        .collect();

    info!("Fetched {} papers", dtos.len());
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_deleted_papers(db: State<'_, Arc<DatabaseConnection>>) -> Result<Vec<PaperDto>> {
    info!("Fetching deleted papers");
    let db = db.inner().as_ref();
    let papers = Papers::find()
        .filter(papers::Column::DeletedAt.is_not_null())
        .order_by_desc(papers::Column::DeletedAt)
        .all(db)
        .await?;

    let authors = papers.load_many_to_many(Authors, PaperAuthors, db).await?;

    let labels = papers.load_many_to_many(Label, PaperLabels, db).await?;

    let attachments = papers.load_many(Attachments, db).await?;

    let dtos: Vec<PaperDto> = papers
        .into_iter()
        .zip(authors.into_iter())
        .zip(labels.into_iter())
        .zip(attachments.into_iter())
        .map(|(((paper, authors), labels), attachments)| PaperDto {
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
            attachment_count: attachments.len(),
            attachments: attachments
                .into_iter()
                .map(|a| AttachmentDto {
                    id: a.id,
                    paper_id: a.paper_id,
                    file_name: a.file_name,
                    file_type: a.file_type,
                    created_at: a.created_at.map(|d| d.to_string()),
                })
                .collect(),
        })
        .collect();

    info!("Fetched {} deleted papers", dtos.len());
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_paper(
    id: i64,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<Option<PaperDetailDto>> {
    info!("Fetching details for paper id {}", id);
    let db = db.inner().as_ref();
    let paper_with_authors = Papers::find_by_id(id)
        .filter(papers::Column::DeletedAt.is_null())
        .find_with_related(Authors)
        .all(db)
        .await?;

    if let Some((paper, authors)) = paper_with_authors.into_iter().next() {
        let labels = paper.find_related(Label).all(db).await?;
        let categories = paper.find_related(Category).all(db).await?;
        let category = categories.first();

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
            labels: labels
                .into_iter()
                .map(|l| LabelDto {
                    id: l.id,
                    name: l.name,
                    color: l.color,
                })
                .collect(),
            category_id: category.map(|c| c.id),
            category_name: category.map(|c| c.name.clone()),
        }))
    } else {
        info!("Paper id {} not found", id);
        Ok(None)
    }
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_paper_details(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    payload: UpdatePaperDto,
) -> Result<()> {
    info!("Updating paper details for id {}", payload.id);
    let db = db.inner().as_ref();

    let paper = Papers::find_by_id(payload.id)
        .filter(papers::Column::DeletedAt.is_null())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", payload.id.to_string()))?;

    let mut active: papers::ActiveModel = paper.into();

    active.title = Set(payload.title.clone());
    active.publication_year = Set(payload.publication_year);
    active.journal_name = Set(payload.journal_name);
    active.conference_name = Set(payload.conference_name);
    active.volume = Set(payload.volume);
    active.issue = Set(payload.issue);
    active.pages = Set(payload.pages);
    active.url = Set(payload.url);
    active.doi = Set(payload.doi);
    active.r#abstract = Set(payload.abstract_text);
    active.notes = Set(payload.notes);
    active.read_status = Set(payload.read_status);

    active.update(db).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Updated")
        .body(format!("Paper '{}' updated successfully", payload.title))
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn delete_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: i64,
) -> Result<()> {
    info!("Soft deleting paper with id {}", id);
    let db = db.inner().as_ref();
    let paper = Papers::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    let mut active: papers::ActiveModel = paper.clone().into();
    active.deleted_at = Set(Some(chrono::Utc::now()));
    active.update(db).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Deleted")
        .body(format!("Paper '{}' moved to trash", paper.title))
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_doi(
    app: AppHandle,
    doi: String,
    category_id: Option<i64>,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<PaperDto> {
    info!("Importing paper with DOI: {}", doi);
    let db = db.inner().as_ref();

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
        .one(db)
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

    // Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(metadata.title.as_bytes());
    let result = hasher.finalize();
    let hash_string = format!("{:x}", result);

    // Create paper
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(Some(metadata.doi.clone())),
        publication_year: Set(publication_year),
        journal_name: Set(metadata.journal_name.clone()),
        url: Set(metadata.url.clone()),
        r#abstract: Set(metadata.abstract_text.clone()),
        attachment_path: Set(Some(hash_string)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Add authors
    for author_name in &metadata.authors {
        // Find or create author
        let author = if let Some(existing_author) = Authors::find()
            .filter(authors::Column::Name.eq(author_name))
            .one(db)
            .await?
        {
            existing_author
        } else {
            authors::ActiveModel {
                name: Set(author_name.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    // Link category if provided
    if let Some(cat_id) = category_id {
        paper_category::ActiveModel {
            paper_id: Set(paper.id),
            category_id: Set(cat_id),
        }
        .insert(db)
        .await?;
    }

    info!(
        "Successfully imported paper: {} (id: {}, doi: {})",
        metadata.title, paper.id, metadata.doi
    );

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
        attachment_count: 0,
        attachments: vec![],
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_arxiv_id(
    app: AppHandle,
    arxiv_id: String,
    category_id: Option<i64>,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<PaperDto> {
    info!("Importing paper with arXiv ID: {}", arxiv_id);
    let db = db.inner().as_ref();

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
            .one(db)
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

    // Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(metadata.title.as_bytes());
    let result = hasher.finalize();
    let hash_string = format!("{:x}", result);

    // Create paper
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(metadata.doi.clone()),
        publication_year: Set(publication_year),
        url: Set(Some(metadata.pdf_url.clone())),
        r#abstract: Set(Some(metadata.summary.clone())),
        journal_name: Set(metadata.journal_ref.clone()),
        attachment_path: Set(Some(hash_string)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Add authors
    for author_name in &metadata.authors {
        // Find or create author
        let author = if let Some(existing_author) = Authors::find()
            .filter(authors::Column::Name.eq(author_name))
            .one(db)
            .await?
        {
            existing_author
        } else {
            authors::ActiveModel {
                name: Set(author_name.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    // Link category if provided
    if let Some(cat_id) = category_id {
        paper_category::ActiveModel {
            paper_id: Set(paper.id),
            category_id: Set(cat_id),
        }
        .insert(db)
        .await?;
    }

    info!(
        "Successfully imported arXiv paper: {} (id: {}, arxiv_id: {})",
        metadata.title, paper.id, metadata.arxiv_id
    );

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
        attachment_count: 0,
        attachments: vec![],
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_pmid(
    app: AppHandle,
    pmid: String,
    category_id: Option<i64>,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<PaperDto> {
    info!("Importing paper with PMID: {}", pmid);
    let db = db.inner().as_ref();

    // Fetch metadata from PubMed using E-utilities API
    let metadata = fetch_pubmed_metadata(&pmid).await.map_err(|e| match e {
        PubmedError::InvalidPmid(id) => {
            AppError::validation("pmid", format!("Invalid PMID: {}", id))
        }
        PubmedError::NotFound => AppError::not_found("PMID", pmid),
        PubmedError::ParseError(msg) => AppError::validation(
            "metadata",
            format!("Failed to parse PubMed metadata: {}", msg),
        ),
        PubmedError::XmlError(msg) => {
            AppError::validation("metadata", format!("Failed to parse PubMed XML: {}", msg))
        }
        PubmedError::RequestError(e) => {
            AppError::network_error(&pmid, format!("Failed to fetch PubMed: {}", e))
        }
    })?;

    // Check if paper already exists by DOI or URL
    if let Some(doi) = &metadata.doi {
        if let Some(existing) = Papers::find()
            .filter(papers::Column::Doi.eq(doi))
            .one(db)
            .await?
        {
            info!("Paper with DOI {} already exists, id: {}", doi, existing.id);
            return Err(AppError::validation(
                "doi",
                format!("Paper with DOI {} already exists", doi),
            ));
        }
    }

    // Store PubMed URL
    let pubmed_url = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", metadata.pmid);

    // Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(metadata.title.as_bytes());
    let result = hasher.finalize();
    let hash_string = format!("{:x}", result);

    // Parse publication year
    let publication_year = metadata
        .publication_year
        .and_then(|y| y.parse::<i64>().ok());

    // Create paper
    let paper = papers::ActiveModel {
        title: Set(metadata.title.clone()),
        doi: Set(metadata.doi.clone()),
        publication_year: Set(publication_year),
        journal_name: Set(metadata.journal_name.clone()),
        url: Set(Some(pubmed_url)),
        r#abstract: Set(metadata.abstract_text.clone()),
        attachment_path: Set(Some(hash_string)),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // Add authors
    for author_name in &metadata.authors {
        // Find or create author
        let author = if let Some(existing_author) = Authors::find()
            .filter(authors::Column::Name.eq(author_name))
            .one(db)
            .await?
        {
            existing_author
        } else {
            authors::ActiveModel {
                name: Set(author_name.clone()),
                ..Default::default()
            }
            .insert(db)
            .await?
        };

        // Link author to paper
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    // Link category if provided
    if let Some(cat_id) = category_id {
        paper_category::ActiveModel {
            paper_id: Set(paper.id),
            category_id: Set(cat_id),
        }
        .insert(db)
        .await?;
    }

    info!(
        "Successfully imported PubMed paper: {} (id: {}, pmid: {})",
        metadata.title, paper.id, metadata.pmid
    );

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
        attachment_count: 0,
        attachments: vec![],
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn add_paper_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: i64,
    label_id: i64,
) -> Result<()> {
    info!("Adding label {} to paper {}", label_id, paper_id);
    let db = db.inner().as_ref();

    // Check if relation already exists to prevent duplicates (though SeaORM might error or ignore)
    // For now, let's just try to insert. If it exists, handle error?
    // Postgres with ON CONFLICT would be nice, but active model insert is basic.

    if (paper_labels::Entity::find_by_id((paper_id, label_id))
        .one(db)
        .await?)
        .is_none()
    {
        paper_labels::ActiveModel {
            paper_id: Set(paper_id),
            label_id: Set(label_id),
        }
        .insert(db)
        .await?;

        let _ = app
            .notification()
            .builder()
            .title("Label Added")
            .body("Label added to paper successfully")
            .show();
    }

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn remove_paper_label(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: i64,
    label_id: i64,
) -> Result<()> {
    info!("Removing label {} from paper {}", label_id, paper_id);
    let db = db.inner().as_ref();
    paper_labels::Entity::delete_by_id((paper_id, label_id))
        .exec(db)
        .await?;

    let _ = app
        .notification()
        .builder()
        .title("Label Removed")
        .body("Label removed from paper successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_papers_by_category(
    db: State<'_, Arc<DatabaseConnection>>,
    category_id: i64,
) -> Result<Vec<PaperDto>> {
    info!("Fetching papers for category id: {}", category_id);
    let db = db.inner().as_ref();

    // Get papers associated with this category
    let papers = Papers::find()
        .inner_join(PaperCategory)
        .filter(paper_category::Column::CategoryId.eq(category_id))
        .filter(papers::Column::DeletedAt.is_null())
        .order_by_desc(papers::Column::Id)
        .all(db)
        .await?;

    let authors = papers.load_many_to_many(Authors, PaperAuthors, db).await?;

    let labels = papers.load_many_to_many(Label, PaperLabels, db).await?;

    let attachments = papers.load_many(Attachments, db).await?;

    let dtos: Vec<PaperDto> = papers
        .into_iter()
        .zip(authors.into_iter())
        .zip(labels.into_iter())
        .zip(attachments.into_iter())
        .map(|(((paper, authors), labels), attachments)| PaperDto {
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
            attachment_count: attachments.len(),
            attachments: attachments
                .into_iter()
                .map(|a| AttachmentDto {
                    id: a.id,
                    paper_id: a.paper_id,
                    file_name: a.file_name,
                    file_type: a.file_type,
                    created_at: a.created_at.map(|d| d.to_string()),
                })
                .collect(),
        })
        .collect();

    info!("Fetched {} papers for category {}", dtos.len(), category_id);
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn update_paper_category(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: i64,
    category_id: Option<i64>,
) -> Result<()> {
    info!(
        "Updating category for paper {}: {:?}",
        paper_id, category_id
    );
    let db = db.inner().as_ref();

    // Transaction?
    let txn = db.begin().await?;

    // 1. Remove all existing categories for this paper (enforce single category)
    paper_category::Entity::delete_many()
        .filter(paper_category::Column::PaperId.eq(paper_id))
        .exec(&txn)
        .await?;

    // 2. Insert new category if provided
    if let Some(cat_id) = category_id {
        paper_category::ActiveModel {
            paper_id: Set(paper_id),
            category_id: Set(cat_id),
        }
        .insert(&txn)
        .await?;
    }

    txn.commit().await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Category Updated")
        .body("Paper category updated successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn restore_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: i64,
) -> Result<()> {
    info!("Restoring paper with id {}", id);
    let db = db.inner().as_ref();
    let paper = Papers::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    let mut active: papers::ActiveModel = paper.into();
    active.deleted_at = Set(None);
    active.update(db).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Restored")
        .body("Paper restored from trash successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn permanently_delete_paper(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    id: i64,
) -> Result<()> {
    info!("Permanently deleting paper with id {}", id);
    let db = db.inner().as_ref();
    let paper = Papers::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    paper.delete(db).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Deleted Permanently")
        .body("Paper permanently deleted successfully")
        .show();

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn add_attachment(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
    file_path: String,
) -> Result<AttachmentDto> {
    info!("Adding attachment for paper {}: {}", paper_id, file_path);
    let db = db.inner().as_ref();

    // 1. Get paper to calculate SHA1 of title
    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // 2. Get or calculate attachment path
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        let hash_string = format!("{:x}", result);

        // Update paper with new attachment path
        let mut active: papers::ActiveModel = paper.clone().into();
        active.attachment_path = Set(Some(hash_string.clone()));
        active.update(db).await?;

        hash_string
    };

    // 3. Create target directory
    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(
                target_dir.to_string_lossy().to_string(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // 4. Copy file
    let source_path = PathBuf::from(&file_path);
    let file_name = source_path
        .file_name()
        .ok_or_else(|| AppError::validation("file_path", "Invalid file path"))?
        .to_string_lossy()
        .to_string();
    let target_path = target_dir.join(&file_name);

    std::fs::copy(&source_path, &target_path).map_err(|e| {
        AppError::file_system(
            target_path.to_string_lossy().to_string(),
            format!("Failed to copy file: {}", e),
        )
    })?;

    // 5. Save to DB
    let attachment = attachments::ActiveModel {
        paper_id: Set(paper_id),
        file_name: Set(Some(file_name.clone())),
        file_type: Set(Some(
            source_path
                .extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default(),
        )),
        created_at: Set(Some(chrono::Utc::now())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let _ = app
        .notification()
        .builder()
        .title("Attachment Added")
        .body("Attachment added successfully")
        .show();

    Ok(AttachmentDto {
        id: attachment.id,
        paper_id: attachment.paper_id,
        file_name: attachment.file_name,
        file_type: attachment.file_type,
        created_at: attachment.created_at.map(|d| d.to_string()),
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_attachments(
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: i64,
) -> Result<Vec<AttachmentDto>> {
    info!("Fetching attachments for paper {}", paper_id);
    let db = db.inner().as_ref();
    let attachments = attachments::Entity::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db)
        .await?;

    Ok(attachments
        .into_iter()
        .map(|a| AttachmentDto {
            id: a.id,
            paper_id: a.paper_id,
            file_name: a.file_name,
            file_type: a.file_type,
            created_at: a.created_at.map(|d| d.to_string()),
        })
        .collect())
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn import_paper_by_pdf(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    category_id: Option<i64>,
) -> Result<PaperDto> {
    info!("Importing paper from PDF: {}", file_path);
    let db = db.inner().as_ref();
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(AppError::file_system(file_path, "File not found"));
    }

    // 1. Get GROBID URL from config
    let config = AppConfig::load(&app_dirs.config)?;
    let grobid_url = config
        .paper
        .grobid
        .servers
        .iter()
        .find(|s| s.is_active)
        .map(|s| s.url.clone())
        .unwrap_or_else(|| "https://kermitt2-grobid.hf.space".to_string());

    // 2. Process with GROBID
    let metadata_result = process_header_document(&path, &grobid_url).await;

    // 3. Prepare paper data
    let (title, metadata) = match metadata_result {
        Ok(m) if !m.title.is_empty() => (m.title.clone(), m),
        _ => {
            info!("GROBID failed or returned empty title, using filename");
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let m = crate::papers::importer::grobid::GrobidMetadata {
                title: filename.clone(),
                ..Default::default()
            };
            (filename, m)
        }
    };

    // 4. Calculate attachment path hash
    let mut hasher = Sha1::new();
    hasher.update(title.as_bytes());
    let result = hasher.finalize();
    let hash_string = format!("{:x}", result);

    // 5. Create paper
    let paper = papers::ActiveModel {
        title: Set(title.clone()),
        doi: Set(metadata.doi),
        publication_year: Set(metadata.publication_year),
        journal_name: Set(metadata.journal_name),
        r#abstract: Set(metadata.abstract_text),
        attachment_path: Set(Some(hash_string.clone())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    // 6. Link authors
    for author_name in metadata.authors {
        let author = if let Some(existing) = Authors::find()
            .filter(authors::Column::Name.eq(&author_name))
            .one(db)
            .await?
        {
            existing
        } else {
            authors::ActiveModel {
                name: Set(author_name),
                ..Default::default()
            }
            .insert(db)
            .await?
        };
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db)
        .await?;
    }

    // 7. Link category
    if let Some(cat_id) = category_id {
        paper_category::ActiveModel {
            paper_id: Set(paper.id),
            category_id: Set(cat_id),
        }
        .insert(db)
        .await?;
    }

    // 8. Copy file to attachment path
    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }
    let target_filename = path.file_name().unwrap().to_string_lossy().to_string();
    let target_path = target_dir.join(&target_filename);
    std::fs::copy(&path, &target_path).map_err(|e| {
        AppError::file_system(target_path.to_string_lossy().to_string(), e.to_string())
    })?;

    // 9. Create attachment record
    attachments::ActiveModel {
        paper_id: Set(paper.id),
        file_name: Set(Some(target_filename.clone())),
        file_type: Set(Some("pdf".to_string())),
        created_at: Set(Some(chrono::Utc::now())),
        ..Default::default()
    }
    .insert(db)
    .await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported from PDF")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: vec![],
        labels: vec![],
        attachment_count: 1,
        attachments: vec![AttachmentDto {
            id: 0, // Placeholder, will be updated if needed
            paper_id: paper.id,
            file_name: Some(target_filename),
            file_type: Some("pdf".to_string()),
            created_at: Some(chrono::Utc::now().to_string()),
        }],
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn open_paper_folder(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
) -> Result<()> {
    info!("Opening folder for paper {}", paper_id);
    let db = db.inner().as_ref();
    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // Get attachment path hash
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    };

    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);

    // Ensure directory exists before opening
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(
                target_dir.to_string_lossy().to_string(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Use opener plugin
    app.opener()
        .open_path(target_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| {
            AppError::file_system(
                target_dir.to_string_lossy().to_string(),
                format!("Failed to open folder: {}", e),
            )
        })?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn get_pdf_attachment_path(
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
) -> Result<PdfAttachmentInfo> {
    info!("Getting PDF attachment path for paper {}", paper_id);
    let db = db.inner().as_ref();

    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // Get attachment path hash (try both lowercase and uppercase)
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    };

    // Try to find PDF attachment in database
    // Get all attachments and filter for PDF (file_type might be "pdf" or "application/pdf")
    let all_attachments = Attachments::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db)
        .await?;

    // Find first attachment that is a PDF (check file_type or file_name extension)
    let attachment = all_attachments
        .iter()
        .find(|a| {
            let file_type = a.file_type.as_deref().unwrap_or("");
            let file_name = a.file_name.as_deref().unwrap_or("");
            file_type.to_lowercase().contains("pdf") || file_name.to_lowercase().ends_with(".pdf")
        })
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;
    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!(
            "{}.pdf",
            paper
                .title
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        )
    });

    // Try lowercase hash first (most common)
    let hash_lower = hash_string.to_lowercase();
    let hash_upper = hash_string.to_uppercase();

    let files_dir = PathBuf::from(&app_dirs.files);

    // Try both lowercase and uppercase hash directories
    let pdf_path = {
        let lower_path = files_dir.join(&hash_lower).join(&file_name);
        let upper_path = files_dir.join(&hash_upper).join(&file_name);

        if lower_path.exists() {
            lower_path
        } else if upper_path.exists() {
            upper_path
        } else {
            // Try to find any PDF in the hash directory
            let lower_dir = files_dir.join(&hash_lower);
            let upper_dir = files_dir.join(&hash_upper);

            if let Ok(Some(found)) = find_first_pdf(&lower_dir) {
                found
            } else if let Ok(Some(found)) = find_first_pdf(&upper_dir) {
                found
            } else {
                return Err(AppError::not_found(
                    "PDF file",
                    format!("hash={}", hash_lower),
                ));
            }
        }
    };

    Ok(PdfAttachmentInfo {
        file_path: pdf_path.to_string_lossy().to_string(),
        file_name,
        paper_id,
        paper_title: paper.title,
        base64_content: None, // Will be populated if requested
    })
}

fn find_first_pdf(dir: &PathBuf) -> Result<Option<PathBuf>> {
    if !dir.exists() {
        return Ok(None);
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| AppError::file_system(dir.to_string_lossy().to_string(), e.to_string()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("pdf") {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2).div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(CHARSET[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARSET[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

fn base64_decode(data: &str) -> std::result::Result<Vec<u8>, String> {
    let bytes = data.as_bytes();
    let mut result = Vec::with_capacity((bytes.len() * 3) / 4);
    let mut i = 0;

    while i < bytes.len() {
        let b1 = base64_char_to_value(bytes[i] as char)?;
        let b2 = if i + 1 < bytes.len() {
            base64_char_to_value(bytes[i + 1] as char)?
        } else {
            0
        };
        let b3 = if i + 2 < bytes.len() {
            base64_char_to_value(bytes[i + 2] as char)?
        } else {
            0
        };
        let b4 = if i + 3 < bytes.len() {
            base64_char_to_value(bytes[i + 3] as char)?
        } else {
            0
        };

        let n = ((b1 as u32) << 18) | ((b2 as u32) << 12) | ((b3 as u32) << 6) | (b4 as u32);

        result.push(((n >> 16) & 0xFF) as u8);
        if bytes[i + 2] as char != '=' {
            result.push(((n >> 8) & 0xFF) as u8);
        }
        if bytes[i + 3] as char != '=' {
            result.push((n & 0xFF) as u8);
        }

        i += 4;
    }

    Ok(result)
}

fn base64_char_to_value(c: char) -> std::result::Result<u8, String> {
    match c {
        'A'..='Z' => Ok(c as u8 - b'A'),
        'a'..='z' => Ok(c as u8 - b'a' + 26),
        '0'..='9' => Ok(c as u8 - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        '=' => Ok(0),
        _ => Err(format!("Invalid base64 character: {}", c)),
    }
}

#[tauri::command]
#[instrument(skip(app_dirs))]
pub async fn read_pdf_file(app_dirs: State<'_, AppDirs>, file_path: String) -> Result<Vec<u8>> {
    info!("Reading PDF file: {}", file_path);

    // Validate the file path is within the app's files directory
    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    // Check if the path is within the allowed directory
    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "file_read: Path {} is not within the allowed directory",
            file_path
        )));
    }

    // Read the file
    let contents = std::fs::read(&path).map_err(|e| {
        AppError::file_system(file_path.clone(), format!("Failed to read file: {}", e))
    })?;

    info!("Successfully read PDF file, size: {} bytes", contents.len());
    Ok(contents)
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn read_pdf_as_blob(
    paper_id: i64,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfBlobResponse> {
    info!("Reading PDF as blob for paper {}", paper_id);
    let db = db.inner().as_ref();

    // Get paper details
    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // Get attachment path hash
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    };

    // Get all attachments and find PDF
    let all_attachments = Attachments::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db)
        .await?;

    let attachment = all_attachments
        .iter()
        .find(|a| {
            let file_type = a.file_type.as_deref().unwrap_or("");
            let file_name = a.file_name.as_deref().unwrap_or("");
            file_type.to_lowercase().contains("pdf") || file_name.to_lowercase().ends_with(".pdf")
        })
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!(
            "{}.pdf",
            paper
                .title
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        )
    });

    // Find PDF file path
    let hash_lower = hash_string.to_lowercase();
    let hash_upper = hash_string.to_uppercase();
    let files_dir = PathBuf::from(&app_dirs.files);

    let pdf_path = {
        let lower_path = files_dir.join(&hash_lower).join(&file_name);
        let upper_path = files_dir.join(&hash_upper).join(&file_name);

        if lower_path.exists() {
            lower_path
        } else if upper_path.exists() {
            upper_path
        } else {
            // Try to find any PDF in the hash directory
            let lower_dir = files_dir.join(&hash_lower);
            let upper_dir = files_dir.join(&hash_upper);

            if let Ok(Some(found)) = find_first_pdf(&lower_dir) {
                found
            } else if let Ok(Some(found)) = find_first_pdf(&upper_dir) {
                found
            } else {
                return Err(AppError::not_found(
                    "PDF file",
                    format!("hash={}", hash_lower),
                ));
            }
        }
    };

    // Read PDF file
    let pdf_bytes = std::fs::read(&pdf_path).map_err(|e| {
        AppError::file_system(
            pdf_path.to_string_lossy().to_string(),
            format!("Failed to read PDF file: {}", e),
        )
    })?;

    let size_bytes = pdf_bytes.len();

    // Encode to base64
    let base64_data = base64_encode(&pdf_bytes);

    info!(
        "Successfully read PDF as blob for paper {}: {} bytes, encoded as base64",
        paper_id, size_bytes
    );

    Ok(PdfBlobResponse {
        file_name,
        paper_title: paper.title,
        paper_id,
        base64_data,
        size_bytes,
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, base64_data, app))]
pub async fn save_pdf_blob(
    app: AppHandle,
    paper_id: i64,
    base64_data: String,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfSaveResponse> {
    info!("Saving PDF blob for paper {}", paper_id);
    let db = db.inner().as_ref();

    // Get paper details
    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // Get attachment path hash
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    };

    // Get all attachments and find PDF
    let all_attachments = Attachments::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db)
        .await?;

    let attachment = all_attachments
        .iter()
        .find(|a| {
            let file_type = a.file_type.as_deref().unwrap_or("");
            let file_name = a.file_name.as_deref().unwrap_or("");
            file_type.to_lowercase().contains("pdf") || file_name.to_lowercase().ends_with(".pdf")
        })
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!(
            "{}.pdf",
            paper
                .title
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        )
    });

    // Decode base64 data
    let pdf_bytes = base64_decode(&base64_data).map_err(|e| {
        AppError::validation("base64_data", format!("Failed to decode base64: {}", e))
    })?;

    let size_bytes = pdf_bytes.len();

    // Find or create PDF file path
    let hash_lower = hash_string.to_lowercase();
    let hash_upper = hash_string.to_uppercase();
    let files_dir = PathBuf::from(&app_dirs.files);

    let pdf_path = {
        let lower_path = files_dir.join(&hash_lower).join(&file_name);
        let upper_path = files_dir.join(&hash_upper).join(&file_name);

        // Check if path already exists
        if lower_path.exists() {
            lower_path
        } else if upper_path.exists() {
            upper_path
        } else {
            // Create in lowercase directory by default
            let lower_dir = files_dir.join(&hash_lower);
            if !lower_dir.exists() {
                std::fs::create_dir_all(&lower_dir).map_err(|e| {
                    AppError::file_system(
                        lower_dir.to_string_lossy().to_string(),
                        format!("Failed to create directory: {}", e),
                    )
                })?;
            }
            lower_path
        }
    };

    // Ensure parent directory exists
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                parent.to_string_lossy().to_string(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write PDF file
    std::fs::write(&pdf_path, &pdf_bytes).map_err(|e| {
        AppError::file_system(
            pdf_path.to_string_lossy().to_string(),
            format!("Failed to write PDF file: {}", e),
        )
    })?;

    info!(
        "Successfully saved PDF blob for paper {}: {} bytes written to {}",
        paper_id,
        size_bytes,
        pdf_path.display()
    );

    let _ = app
        .notification()
        .builder()
        .title("PDF Saved")
        .body("PDF saved successfully")
        .show();

    Ok(PdfSaveResponse {
        success: true,
        file_path: pdf_path.to_string_lossy().to_string(),
        size_bytes,
        message: format!(
            "PDF saved successfully: {} ({} bytes)",
            file_name, size_bytes
        ),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, base64_data, app))]
pub async fn save_pdf_with_annotations(
    app: AppHandle,
    paper_id: i64,
    base64_data: String,
    annotations_json: Option<String>,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfSaveResponse> {
    info!("Saving PDF blob with annotations for paper {}", paper_id);
    let db = db.inner().as_ref();

    // Get paper details
    let paper = Papers::find_by_id(paper_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.to_string()))?;

    // Get attachment path hash
    let hash_string = if let Some(path) = &paper.attachment_path {
        path.clone()
    } else {
        let mut hasher = Sha1::new();
        hasher.update(paper.title.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    };

    // Get all attachments and find PDF
    let all_attachments = Attachments::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db)
        .await?;

    let attachment = all_attachments
        .iter()
        .find(|a| {
            let file_type = a.file_type.as_deref().unwrap_or("");
            let file_name = a.file_name.as_deref().unwrap_or("");
            file_type.to_lowercase().contains("pdf") || file_name.to_lowercase().ends_with(".pdf")
        })
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!(
            "{}.pdf",
            paper
                .title
                .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        )
    });

    // Decode base64 data
    let pdf_bytes = base64_decode(&base64_data).map_err(|e| {
        AppError::validation("base64_data", format!("Failed to decode base64: {}", e))
    })?;

    let size_bytes = pdf_bytes.len();

    // Find or create PDF file path
    let hash_lower = hash_string.to_lowercase();
    let hash_upper = hash_string.to_uppercase();
    let files_dir = PathBuf::from(&app_dirs.files);

    let lower_path = files_dir.join(&hash_lower).join(&file_name);
    let upper_path = files_dir.join(&hash_upper).join(&file_name);

    let pdf_path = if lower_path.exists() {
        lower_path
    } else if upper_path.exists() {
        upper_path
    } else {
        let lower_dir = files_dir.join(&hash_lower);
        if !lower_dir.exists() {
            std::fs::create_dir_all(&lower_dir).map_err(|e| {
                AppError::file_system(
                    lower_dir.to_string_lossy().to_string(),
                    format!("Failed to create directory: {}", e),
                )
            })?;
        }
        lower_path
    };

    // Ensure parent directory exists
    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                parent.to_string_lossy().to_string(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write PDF file
    std::fs::write(&pdf_path, &pdf_bytes).map_err(|e| {
        AppError::file_system(
            pdf_path.to_string_lossy().to_string(),
            format!("Failed to write PDF file: {}", e),
        )
    })?;

    // Save annotations if provided
    if let Some(annotations) = annotations_json {
        info!("Saving annotations for paper {}", paper_id);
        let annotations_path = pdf_path.with_extension("json");

        std::fs::write(&annotations_path, &annotations).map_err(|e| {
            AppError::file_system(
                annotations_path.to_string_lossy().to_string(),
                format!("Failed to write annotations file: {}", e),
            )
        })?;

        info!(
            "Successfully saved PDF with annotations for paper {}: {} bytes written to {}",
            paper_id,
            size_bytes,
            pdf_path.display()
        );

        let _ = app
            .notification()
            .builder()
            .title("Annotations Saved")
            .body("PDF and annotations saved successfully")
            .show();

        return Ok(PdfSaveResponse {
            success: true,
            file_path: pdf_path.to_string_lossy().to_string(),
            size_bytes,
            message: format!(
                "PDF and annotations saved successfully ({} bytes)",
                size_bytes
            ),
        });
    }

    return Err(AppError::DocumentParseError {
        message: format!(
            "PDF saved successfully: {} ({} bytes)",
            &file_name, size_bytes
        ),
    });
}
