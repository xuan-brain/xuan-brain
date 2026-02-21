//! Import operations for papers (DOI, arXiv, PMID, PDF)

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::papers::importer::grobid::process_header_document;
use crate::papers::importer::pubmed::{fetch_pubmed_metadata, PubmedError};
use crate::repository::{AttachmentRepository, AuthorRepository, PaperRepository};
use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateAttachment, CreatePaper};
use crate::sys::config::AppConfig;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::{calculate_attachment_hash, record_id_to_string};

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_doi(
    app: AppHandle,
    doi: String,
    category_id: Option<String>,
    db: State<'_, Arc<SurrealClient>>,
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

    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);

    // Check if paper already exists
    if let Some(_) = paper_repo.find_by_doi(&metadata.doi).await? {
        return Err(AppError::validation(
            "doi",
            format!("Paper with DOI {} already exists", metadata.doi),
        ));
    }

    // Calculate attachment path hash
    let hash_string = calculate_attachment_hash(&metadata.title);

    // Create paper
    let publication_year = metadata.publication_year.and_then(|y| y.parse::<i32>().ok());

    let paper = paper_repo.create(CreatePaper {
        title: metadata.title.clone(),
        doi: Some(metadata.doi.clone()),
        publication_year,
        publication_date: None,
        journal_name: metadata.journal_name.clone(),
        conference_name: None,
        volume: None,
        issue: None,
        pages: None,
        url: metadata.url.clone(),
        abstract_text: metadata.abstract_text.clone(),
        attachment_path: Some(hash_string),
    }).await?;

    let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

    // Add authors
    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = author_repo.create_or_find(author_name, None).await?;
        let author_id = author.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();
        paper_repo.add_author(&paper_id, &author_id, order as i32).await?;
    }

    // Link category if provided
    if let Some(cat_id) = category_id {
        paper_repo.set_category(&paper_id, Some(cat_id)).await?;
    }

    info!("Successfully imported paper: {} (doi: {})", metadata.title, metadata.doi);

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper_id,
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
    category_id: Option<String>,
    db: State<'_, Arc<SurrealClient>>,
) -> Result<PaperDto> {
    info!("Importing paper with arXiv ID: {}", arxiv_id);

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

    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);

    // Check if paper already exists by DOI
    if let Some(doi) = &metadata.doi {
        if let Some(_) = paper_repo.find_by_doi(doi).await? {
            return Err(AppError::validation(
                "doi",
                format!("Paper with DOI {} already exists", doi),
            ));
        }
    }

    let hash_string = calculate_attachment_hash(&metadata.title);
    let publication_year = metadata.published.split('-').next().and_then(|y| y.parse::<i32>().ok());

    let paper = paper_repo.create(CreatePaper {
        title: metadata.title.clone(),
        doi: metadata.doi.clone(),
        publication_year,
        publication_date: Some(metadata.published.clone()),
        journal_name: metadata.journal_ref.clone(),
        conference_name: None,
        volume: None,
        issue: None,
        pages: None,
        url: Some(metadata.pdf_url.clone()),
        abstract_text: Some(metadata.summary.clone()),
        attachment_path: Some(hash_string),
    }).await?;

    let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = author_repo.create_or_find(author_name, None).await?;
        let author_id = author.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();
        paper_repo.add_author(&paper_id, &author_id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        paper_repo.set_category(&paper_id, Some(cat_id)).await?;
    }

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper_id,
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
    category_id: Option<String>,
    db: State<'_, Arc<SurrealClient>>,
) -> Result<PaperDto> {
    info!("Importing paper with PMID: {}", pmid);

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

    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);

    if let Some(doi) = &metadata.doi {
        if let Some(_) = paper_repo.find_by_doi(doi).await? {
            return Err(AppError::validation(
                "doi",
                format!("Paper with DOI {} already exists", doi),
            ));
        }
    }

    let pubmed_url = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", metadata.pmid);
    let hash_string = calculate_attachment_hash(&metadata.title);
    let publication_year = metadata.publication_year.and_then(|y| y.parse::<i32>().ok());

    let paper = paper_repo.create(CreatePaper {
        title: metadata.title.clone(),
        doi: metadata.doi.clone(),
        publication_year,
        publication_date: None,
        journal_name: metadata.journal_name.clone(),
        conference_name: None,
        volume: None,
        issue: None,
        pages: None,
        url: Some(pubmed_url),
        abstract_text: metadata.abstract_text.clone(),
        attachment_path: Some(hash_string),
    }).await?;

    let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = author_repo.create_or_find(author_name, None).await?;
        let author_id = author.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();
        paper_repo.add_author(&paper_id, &author_id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        paper_repo.set_category(&paper_id, Some(cat_id)).await?;
    }

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper_id,
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
#[instrument(skip(db, app_dirs, app))]
pub async fn import_paper_by_pdf(
    app: AppHandle,
    db: State<'_, Arc<SurrealClient>>,
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    category_id: Option<String>,
) -> Result<PaperDto> {
    info!("Importing paper from PDF: {}", file_path);
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(AppError::file_system(file_path, "File not found"));
    }

    // Get GROBID URL from config
    let config = AppConfig::load(&app_dirs.config)?;
    let grobid_url = config
        .paper
        .grobid
        .servers
        .iter()
        .find(|s| s.is_active)
        .map(|s| s.url.clone())
        .unwrap_or_else(|| "https://kermitt2-grobid.hf.space".to_string());

    let metadata_result = process_header_document(&path, &grobid_url).await;

    let (title, metadata) = match metadata_result {
        Ok(m) if !m.title.is_empty() => (m.title.clone(), m),
        _ => {
            let filename = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
            let m = crate::papers::importer::grobid::GrobidMetadata {
                title: filename.clone(),
                ..Default::default()
            };
            (filename, m)
        }
    };

    let paper_repo = PaperRepository::new(&db);
    let author_repo = AuthorRepository::new(&db);
    let attachment_repo = AttachmentRepository::new(&db);

    let hash_string = calculate_attachment_hash(&title);

    let paper = paper_repo.create(CreatePaper {
        title: title.clone(),
        doi: metadata.doi.clone(),
        publication_year: metadata.publication_year.and_then(|y| i32::try_from(y).ok()),
        publication_date: None,
        journal_name: metadata.journal_name.clone(),
        conference_name: None,
        volume: None,
        issue: None,
        pages: None,
        url: None,
        abstract_text: metadata.abstract_text.clone(),
        attachment_path: Some(hash_string.clone()),
    }).await?;

    let paper_id = paper.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();

    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = author_repo.create_or_find(author_name, None).await?;
        let author_id = author.id.as_ref().map(|rid| record_id_to_string(rid)).unwrap_or_default();
        paper_repo.add_author(&paper_id, &author_id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        paper_repo.set_category(&paper_id, Some(cat_id)).await?;
    }

    // Copy file to attachment path
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

    // Create attachment record
    attachment_repo.create(CreateAttachment {
        paper_id: paper_id.clone(),
        file_name: Some(target_filename.clone()),
        file_type: Some("pdf".to_string()),
        file_path: Some(target_path.to_string_lossy().to_string()),
        file_size: None,
    }).await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported from PDF")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(PaperDto {
        id: paper_id.clone(),
        title: paper.title,
        publication_year: paper.publication_year,
        journal_name: paper.journal_name,
        conference_name: paper.conference_name,
        authors: metadata.authors,
        labels: vec![],
        attachment_count: 1,
        attachments: vec![AttachmentDto {
            id: String::new(),
            paper_id,
            file_name: Some(target_filename),
            file_type: Some("pdf".to_string()),
            created_at: None,
        }],
    })
}
