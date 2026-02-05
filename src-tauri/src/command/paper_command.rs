use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, LoaderTrait, ModelTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use tracing::{info, instrument};

use crate::database::entities::{
    attachments, authors, category, paper_authors, paper_category, paper_labels, papers, prelude::*,
};
use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::papers::importer::grobid::process_header_document;
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
    pub category_path: Option<String>,
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
pub async fn get_all_papers(db: State<'_, DatabaseConnection>) -> Result<Vec<PaperDto>> {
    info!("Fetching all papers");
    let papers = Papers::find()
        .filter(papers::Column::DeletedAt.is_null())
        .order_by_desc(papers::Column::Id)
        .all(db.inner())
        .await?;

    let authors = papers
        .load_many_to_many(Authors, PaperAuthors, db.inner())
        .await?;

    let labels = papers
        .load_many_to_many(Label, PaperLabels, db.inner())
        .await?;

    let attachments = papers.load_many(Attachments, db.inner()).await?;

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
        })
        .collect();

    info!("Fetched {} papers", dtos.len());
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_deleted_papers(db: State<'_, DatabaseConnection>) -> Result<Vec<PaperDto>> {
    info!("Fetching deleted papers");
    let papers = Papers::find()
        .filter(papers::Column::DeletedAt.is_not_null())
        .order_by_desc(papers::Column::DeletedAt)
        .all(db.inner())
        .await?;

    let authors = papers
        .load_many_to_many(Authors, PaperAuthors, db.inner())
        .await?;

    let labels = papers
        .load_many_to_many(Label, PaperLabels, db.inner())
        .await?;

    let attachments = papers.load_many(Attachments, db.inner()).await?;

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
        })
        .collect();

    info!("Fetched {} deleted papers", dtos.len());
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
        .filter(papers::Column::DeletedAt.is_null())
        .find_with_related(Authors)
        .all(db.inner())
        .await?;

    if let Some((paper, authors)) = paper_with_authors.into_iter().next() {
        let labels = paper.find_related(Label).all(db.inner()).await?;
        let categories = paper.find_related(Category).all(db.inner()).await?;
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
            category_path: category.map(|c| c.ltree_path.clone()),
        }))
    } else {
        info!("Paper id {} not found", id);
        Ok(None)
    }
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn update_paper_details(
    db: State<'_, DatabaseConnection>,
    payload: UpdatePaperDto,
) -> Result<()> {
    info!("Updating paper details for id {}", payload.id);

    let paper = Papers::find_by_id(payload.id)
        .filter(papers::Column::DeletedAt.is_null())
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Paper", payload.id.to_string()))?;

    let mut active: papers::ActiveModel = paper.into();

    active.title = Set(payload.title);
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

    active.update(db.inner()).await?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_paper(db: State<'_, DatabaseConnection>, id: i64) -> Result<()> {
    info!("Soft deleting paper with id {}", id);
    let paper = Papers::find_by_id(id)
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    let mut active: papers::ActiveModel = paper.into();
    active.deleted_at = Set(Some(chrono::Utc::now()));
    active.update(db.inner()).await?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_doi(
    doi: String,
    category_path: Option<String>,
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

    // Link category if provided
    if let Some(path) = category_path {
        if let Some(cat) = Category::find()
            .filter(category::Column::LtreePath.eq(&path))
            .one(db.inner())
            .await?
        {
            paper_category::ActiveModel {
                paper_id: Set(paper.id),
                category_id: Set(cat.id),
            }
            .insert(db.inner())
            .await?;
        }
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
        attachment_count: 0,
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_arxiv_id(
    arxiv_id: String,
    category_path: Option<String>,
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

    // Link category if provided
    if let Some(path) = category_path {
        if let Some(cat) = Category::find()
            .filter(category::Column::LtreePath.eq(&path))
            .one(db.inner())
            .await?
        {
            paper_category::ActiveModel {
                paper_id: Set(paper.id),
                category_id: Set(cat.id),
            }
            .insert(db.inner())
            .await?;
        }
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
        attachment_count: 0,
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn add_paper_label(
    db: State<'_, DatabaseConnection>,
    paper_id: i64,
    label_id: i64,
) -> Result<()> {
    info!("Adding label {} to paper {}", label_id, paper_id);

    // Check if relation already exists to prevent duplicates (though SeaORM might error or ignore)
    // For now, let's just try to insert. If it exists, handle error?
    // Postgres with ON CONFLICT would be nice, but active model insert is basic.

    if (paper_labels::Entity::find_by_id((paper_id, label_id))
        .one(db.inner())
        .await?)
        .is_none()
    {
        paper_labels::ActiveModel {
            paper_id: Set(paper_id),
            label_id: Set(label_id),
        }
        .insert(db.inner())
        .await?;
    }

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn remove_paper_label(
    db: State<'_, DatabaseConnection>,
    paper_id: i64,
    label_id: i64,
) -> Result<()> {
    info!("Removing label {} from paper {}", label_id, paper_id);
    paper_labels::Entity::delete_by_id((paper_id, label_id))
        .exec(db.inner())
        .await?;
    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_papers_by_category(
    db: State<'_, DatabaseConnection>,
    category_path: String,
) -> Result<Vec<PaperDto>> {
    info!("Fetching papers for category path: {}", category_path);

    // First, get the category ID by path
    let category_entity = Category::find()
        .filter(category::Column::LtreePath.eq(&category_path))
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Category", category_path.clone()))?;

    // Then get papers associated with this category
    let papers = Papers::find()
        .inner_join(PaperCategory)
        .filter(paper_category::Column::CategoryId.eq(category_entity.id))
        .filter(papers::Column::DeletedAt.is_null())
        .order_by_desc(papers::Column::Id)
        .all(db.inner())
        .await?;

    let authors = papers
        .load_many_to_many(Authors, PaperAuthors, db.inner())
        .await?;

    let labels = papers
        .load_many_to_many(Label, PaperLabels, db.inner())
        .await?;

    let attachments = papers.load_many(Attachments, db.inner()).await?;

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
        })
        .collect();

    info!(
        "Fetched {} papers for category {}",
        dtos.len(),
        category_path
    );
    Ok(dtos)
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn update_paper_category(
    db: State<'_, DatabaseConnection>,
    paper_id: i64,
    category_id: Option<i64>,
) -> Result<()> {
    info!(
        "Updating category for paper {}: {:?}",
        paper_id, category_id
    );

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

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn restore_paper(db: State<'_, DatabaseConnection>, id: i64) -> Result<()> {
    info!("Restoring paper with id {}", id);
    let paper = Papers::find_by_id(id)
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    let mut active: papers::ActiveModel = paper.into();
    active.deleted_at = Set(None);
    active.update(db.inner()).await?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn permanently_delete_paper(db: State<'_, DatabaseConnection>, id: i64) -> Result<()> {
    info!("Permanently deleting paper with id {}", id);
    let paper = Papers::find_by_id(id)
        .one(db.inner())
        .await?
        .ok_or_else(|| AppError::not_found("Paper", id.to_string()))?;

    paper.delete(db.inner()).await?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn add_attachment(
    db: State<'_, DatabaseConnection>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
    file_path: String,
) -> Result<AttachmentDto> {
    info!("Adding attachment for paper {}: {}", paper_id, file_path);

    // 1. Get paper to calculate SHA1 of title
    let paper = Papers::find_by_id(paper_id)
        .one(db.inner())
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
        active.update(db.inner()).await?;

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
    .insert(db.inner())
    .await?;

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
    db: State<'_, DatabaseConnection>,
    paper_id: i64,
) -> Result<Vec<AttachmentDto>> {
    info!("Fetching attachments for paper {}", paper_id);
    let attachments = attachments::Entity::find()
        .filter(attachments::Column::PaperId.eq(paper_id))
        .all(db.inner())
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
#[instrument(skip(db, app_dirs))]
pub async fn import_paper_by_pdf(
    db: State<'_, DatabaseConnection>,
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    category_path: Option<String>,
) -> Result<PaperDto> {
    info!("Importing paper from PDF: {}", file_path);
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
            let mut m = crate::papers::importer::grobid::GrobidMetadata::default();
            m.title = filename.clone();
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
    .insert(db.inner())
    .await?;

    // 6. Link authors
    for author_name in metadata.authors {
        let author = if let Some(existing) = Authors::find()
            .filter(authors::Column::Name.eq(&author_name))
            .one(db.inner())
            .await?
        {
            existing
        } else {
            authors::ActiveModel {
                name: Set(author_name),
                ..Default::default()
            }
            .insert(db.inner())
            .await?
        };
        paper_authors::ActiveModel {
            paper_id: Set(paper.id),
            author_id: Set(author.id),
            ..Default::default()
        }
        .insert(db.inner())
        .await?;
    }

    // 7. Link category
    if let Some(path_str) = category_path {
        if let Some(cat) = Category::find()
            .filter(category::Column::LtreePath.eq(&path_str))
            .one(db.inner())
            .await?
        {
            paper_category::ActiveModel {
                paper_id: Set(paper.id),
                category_id: Set(cat.id),
            }
            .insert(db.inner())
            .await?;
        }
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
        file_name: Set(Some(target_filename)),
        file_type: Set(Some("pdf".to_string())),
        created_at: Set(Some(chrono::Utc::now())),
        ..Default::default()
    }
    .insert(db.inner())
    .await?;

    Ok(PaperDto {
        id: paper.id,
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: vec![],
        labels: vec![],
        attachment_count: 1,
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn open_paper_folder(
    app: AppHandle,
    db: State<'_, DatabaseConnection>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
) -> Result<()> {
    info!("Opening folder for paper {}", paper_id);
    let paper = Papers::find_by_id(paper_id)
        .one(db.inner())
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
    db: State<'_, DatabaseConnection>,
    app_dirs: State<'_, AppDirs>,
    paper_id: i64,
) -> Result<PdfAttachmentInfo> {
    info!("Getting PDF attachment path for paper {}", paper_id);

    let paper = Papers::find_by_id(paper_id)
        .one(db.inner())
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
        .all(db.inner())
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
#[instrument(skip(app_dirs))]
pub async fn save_pdf_file(
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    pdf_data: Vec<u8>,
) -> Result<()> {
    info!(
        "Saving PDF file: {}, size: {} bytes",
        file_path,
        pdf_data.len()
    );

    // Validate the file path is within the app's files directory
    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    // Check if the path is within the allowed directory
    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "save_pdf_file: Path {} is not within the allowed directory",
            file_path
        )));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                file_path.clone(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write the file
    std::fs::write(&path, &pdf_data).map_err(|e| {
        AppError::file_system(file_path.clone(), format!("Failed to write file: {}", e))
    })?;

    info!("Successfully saved PDF file: {}", file_path);
    Ok(())
}

#[tauri::command]
#[instrument(skip(app_dirs))]
pub async fn save_annotations_data(
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    annotations_json: String,
) -> Result<()> {
    info!(
        "Saving annotations data for file: {}, size: {} bytes",
        file_path,
        annotations_json.len()
    );

    // Validate the file path is within the app's files directory
    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    // Check if the path is within the allowed directory
    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "save_annotations_data: Path {} is not within the allowed directory",
            file_path
        )));
    }

    // Save annotations to a sidecar .json file
    let annotations_path = path.with_extension("json");
    std::fs::write(&annotations_path, &annotations_json).map_err(|e| {
        AppError::file_system(
            annotations_path.to_string_lossy().to_string(),
            format!("Failed to write annotations file: {}", e),
        )
    })?;

    info!(
        "Successfully saved annotations data to: {}",
        annotations_path.display()
    );
    Ok(())
}

#[tauri::command]
#[instrument(skip(app_dirs))]
pub async fn load_annotations_data(
    app_dirs: State<'_, AppDirs>,
    file_path: String,
) -> Result<Option<String>> {
    info!("Loading annotations data for file: {}", file_path);

    // Validate the file path is within the app's files directory
    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    // Check if the path is within the allowed directory
    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "load_annotations_data: Path {} is not within the allowed directory",
            file_path
        )));
    }

    // Try to load annotations from sidecar .json file
    let annotations_path = path.with_extension("json");
    match std::fs::read_to_string(&annotations_path) {
        Ok(content) => {
            info!(
                "Successfully loaded annotations data from: {}",
                annotations_path.display()
            );
            Ok(Some(content))
        }
        Err(_) => {
            info!(
                "No annotations data found at: {}",
                annotations_path.display()
            );
            Ok(None)
        }
    }
}

#[tauri::command]
#[instrument(skip(app_dirs))]
pub async fn export_pdf_with_annotations(
    app_dirs: State<'_, AppDirs>,
    source_file_path: String,
    export_file_path: String,
    pdf_data: Vec<u8>,
) -> Result<()> {
    info!(
        "Exporting PDF with annotations from {} to {}",
        source_file_path, export_file_path
    );

    // Validate both file paths are within the app's allowed directories
    let source_path = PathBuf::from(&source_file_path);
    let export_path = PathBuf::from(&export_file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    if !source_path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "export_pdf_with_annotations: Source path {} is not within the allowed directory",
            source_file_path
        )));
    }

    if !export_path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "export_pdf_with_annotations: Export path {} is not within the allowed directory",
            export_file_path
        )));
    }

    // Ensure export directory exists
    if let Some(parent) = export_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                export_file_path.clone(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write the exported file
    std::fs::write(&export_path, &pdf_data).map_err(|e| {
        AppError::file_system(
            export_file_path.clone(),
            format!("Failed to write exported file: {}", e),
        )
    })?;

    info!(
        "Successfully saved PDF with annotations to: {}",
        export_file_path
    );
    Ok(())
}

#[tauri::command]
#[instrument(skip(app_dirs, pdf_data))]
pub async fn save_pdf_with_annotations_data(
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    pdf_data: Vec<u8>,
    annotations_json: String,
) -> Result<()> {
    info!(
        "Saving PDF with annotations: {}, size: {} bytes",
        file_path,
        pdf_data.len()
    );

    // Validate the file path is within the app's files directory
    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    // Check if the path is within the allowed directory
    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "save_pdf_with_annotations_data: Path {} is not within the allowed directory",
            file_path
        )));
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(
                file_path.clone(),
                format!("Failed to create directory: {}", e),
            )
        })?;
    }

    // Write the file directly (annotations are saved separately as JSON)
    // This is a workaround since we don't have a Rust PDF library for merging annotations
    std::fs::write(&path, &pdf_data).map_err(|e| {
        AppError::file_system(file_path.clone(), format!("Failed to write file: {}", e))
    })?;

    // Save annotations as JSON sidecar file
    let annotations_path = path.with_extension("json");
    std::fs::write(&annotations_path, &annotations_json).map_err(|e| {
        AppError::file_system(
            annotations_path.to_string_lossy().to_string(),
            format!("Failed to write annotations file: {}", e),
        )
    })?;

    info!(
        "Successfully saved PDF with annotations: {} (annotations: {})",
        file_path,
        annotations_path.display()
    );
    Ok(())
}
