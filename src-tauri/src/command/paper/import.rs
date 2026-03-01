//! Import operations for papers (DOI, arXiv, PMID, PDF)

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::CreatePaper;
use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::papers::importer::grobid::process_header_document;
use crate::papers::importer::pubmed::{fetch_pubmed_metadata, PubmedError};
use crate::repository::{AuthorRepository, PaperRepository};
use crate::sys::config::AppConfig;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::calculate_attachment_hash;

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_doi(
    app: AppHandle,
    doi: String,
    category_id: Option<String>,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<ImportResultDto> {
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
    if let Some(existing_paper) = PaperRepository::find_by_doi(&db, &metadata.doi).await? {
        info!("Paper with DOI {} already exists: {}", metadata.doi, existing_paper.title);
        let _ = app
            .notification()
            .builder()
            .title("Paper Already Exists")
            .body(format!("Paper '{}' is already in your library", existing_paper.title))
            .show();

        return Ok(ImportResultDto {
            already_exists: true,
            message: format!("Paper '{}' is already in your library", existing_paper.title),
            paper: None,
        });
    }

    // Calculate attachment path hash
    let hash_string = calculate_attachment_hash(&metadata.title);

    // Create paper
    let publication_year = metadata.publication_year.and_then(|y| y.parse::<i32>().ok());

    let paper = PaperRepository::create(
        &db,
        CreatePaper {
            title: metadata.title.clone(),
            doi: Some(metadata.doi.clone()),
            publication_year,
            publication_date: None,
            journal_name: metadata.journal_name.clone(),
            conference_name: None,
            volume: metadata.volume.clone(),
            issue: metadata.issue.clone(),
            pages: metadata.pages.clone(),
            url: metadata.url.clone(),
            abstract_text: metadata.abstract_text.clone(),
            attachment_path: Some(hash_string),
        },
    )
    .await?;

    let paper_id = paper.id;

    // Add authors and create paper-author relations
    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find(&db, author_name, None).await?;
        // Create paper-author relation
        PaperRepository::add_author(&db, paper_id, author.id, order as i32).await?;
    }

    // Link category if provided
    if let Some(cat_id) = category_id {
        let cat_id_num = cat_id
            .parse::<i64>()
            .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;
        PaperRepository::set_category(&db, paper_id, Some(cat_id_num)).await?;
    }

    info!(
        "Successfully imported paper: {} (doi: {})",
        metadata.title, metadata.doi
    );

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: metadata.authors,
            labels: vec![],
            attachment_count: 0,
            attachments: vec![],
        }),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn import_paper_by_arxiv_id(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    arxiv_id: String,
    category_id: Option<String>,
) -> Result<ImportResultDto> {
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

    // Check if paper already exists by DOI
    if let Some(doi) = &metadata.doi {
        if let Some(existing_paper) = PaperRepository::find_by_doi(&db, doi).await? {
            info!("Paper with DOI {} already exists: {}", doi, existing_paper.title);
            let _ = app
                .notification()
                .builder()
                .title("Paper Already Exists")
                .body(format!("Paper '{}' is already in your library", existing_paper.title))
                .show();

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!("Paper '{}' is already in your library", existing_paper.title),
                paper: None,
            });
        }
    }

    let hash_string = calculate_attachment_hash(&metadata.title);
    let publication_year = metadata
        .published
        .split('-')
        .next()
        .and_then(|y| y.parse::<i32>().ok());

    let paper = PaperRepository::create(
        &db,
        CreatePaper {
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
            attachment_path: Some(hash_string.clone()),
        },
    )
    .await?;

    let paper_id = paper.id;

    // Add authors and create paper-author relations
    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find(&db, author_name, None).await?;
        // Create paper-author relation
        PaperRepository::add_author(&db, paper_id, author.id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        let cat_id_num = cat_id
            .parse::<i64>()
            .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;
        PaperRepository::set_category(&db, paper_id, Some(cat_id_num)).await?;
    }

    // Download PDF from arXiv
    let pdf_filename = format!("{}.pdf", metadata.arxiv_id.replace('/', "_"));
    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }
    let target_path = target_dir.join(&pdf_filename);

    info!("Downloading arXiv PDF from: {}", metadata.pdf_url);
    info!("Saving to: {:?}", target_path);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120)) // 2 minutes timeout for large PDFs
        .build()
        .map_err(|e| AppError::network_error(&metadata.pdf_url, format!("Failed to create HTTP client: {}", e)))?;

    let response = client
        .get(&metadata.pdf_url)
        .send()
        .await
        .map_err(|e| AppError::network_error(&metadata.pdf_url, format!("Failed to download PDF: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::network_error(
            &metadata.pdf_url,
            format!("Failed to download PDF: HTTP {}", response.status()),
        ));
    }

    let pdf_bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::network_error(&metadata.pdf_url, format!("Failed to read PDF content: {}", e)))?;

    std::fs::write(&target_path, &pdf_bytes).map_err(|e| {
        AppError::file_system(target_path.to_string_lossy().to_string(), e.to_string())
    })?;

    info!("PDF downloaded successfully: {} bytes", pdf_bytes.len());

    // Create attachment record
    let file_size = Some(pdf_bytes.len() as i64);
    PaperRepository::add_attachment(
        &db,
        paper_id,
        Some(pdf_filename.clone()),
        Some("pdf".to_string()),
        file_size,
    )
    .await?;

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: metadata.authors,
            labels: vec![],
            attachment_count: 1,
            attachments: vec![AttachmentDto {
                id: String::new(),
                paper_id: paper_id.to_string(),
                file_name: Some(pdf_filename),
                file_type: Some("pdf".to_string()),
                created_at: None,
            }],
        }),
    })
}

#[tauri::command]
#[instrument(skip(db, app))]
pub async fn import_paper_by_pmid(
    app: AppHandle,
    pmid: String,
    category_id: Option<String>,
    db: State<'_, Arc<DatabaseConnection>>,
) -> Result<ImportResultDto> {
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

    if let Some(doi) = &metadata.doi {
        if let Some(existing_paper) = PaperRepository::find_by_doi(&db, doi).await? {
            info!("Paper with DOI {} already exists: {}", doi, existing_paper.title);
            let _ = app
                .notification()
                .builder()
                .title("Paper Already Exists")
                .body(format!("Paper '{}' is already in your library", existing_paper.title))
                .show();

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!("Paper '{}' is already in your library", existing_paper.title),
                paper: None,
            });
        }
    }

    let pubmed_url = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", metadata.pmid);
    let hash_string = calculate_attachment_hash(&metadata.title);
    let publication_year = metadata.publication_year.and_then(|y| y.parse::<i32>().ok());

    let paper = PaperRepository::create(
        &db,
        CreatePaper {
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
        },
    )
    .await?;

    let paper_id = paper.id;

    // Add authors and create paper-author relations
    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find(&db, author_name, None).await?;
        // Create paper-author relation
        PaperRepository::add_author(&db, paper_id, author.id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        let cat_id_num = cat_id
            .parse::<i64>()
            .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;
        PaperRepository::set_category(&db, paper_id, Some(cat_id_num)).await?;
    }

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: metadata.authors,
            labels: vec![],
            attachment_count: 0,
            attachments: vec![],
        }),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn import_paper_by_pdf(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    category_id: Option<String>,
) -> Result<ImportResultDto> {
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

    info!("Using GROBID server: {}", grobid_url);

    // Try to get metadata from GROBID, but don't fail the whole import if it fails
    let metadata_result = process_header_document(&path, &grobid_url).await;

    let (title, metadata) = match metadata_result {
        Ok(m) if !m.title.is_empty() => {
            info!("Successfully extracted metadata from GROBID");
            (m.title.clone(), m)
        }
        Ok(m) => {
            info!("GROBID returned empty title, using filename");
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let m = crate::papers::importer::grobid::GrobidMetadata {
                title: filename.clone(),
                ..m
            };
            (filename, m)
        }
        Err(e) => {
            info!("GROBID extraction failed: {}, using filename as title", e);
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

    info!("Using title: {}", title);

    // Check if paper already exists by DOI (if available)
    if let Some(ref doi) = metadata.doi {
        if let Some(existing_paper) = PaperRepository::find_by_doi(&db, doi).await? {
            info!("Paper with DOI {} already exists: {}", doi, existing_paper.title);
            let _ = app
                .notification()
                .builder()
                .title("Paper Already Exists")
                .body(format!("Paper '{}' is already in your library", existing_paper.title))
                .show();

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!("Paper '{}' is already in your library", existing_paper.title),
                paper: None,
            });
        }
    }

    let target_filename = path.file_name().unwrap().to_string_lossy().to_string();
    let hash_string = calculate_attachment_hash(&title);

    info!("Creating paper record with hash: {}", hash_string);

    let paper = PaperRepository::create(
        &db,
        CreatePaper {
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
        },
    )
    .await?;

    let paper_id = paper.id;
    info!("Created paper with ID: {}", paper_id);

    // Add authors and create paper-author relations
    for (order, author_name) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find(&db, author_name, None).await?;
        // Create paper-author relation
        PaperRepository::add_author(&db, paper_id, author.id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        let cat_id_num = cat_id
            .parse::<i64>()
            .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;
        PaperRepository::set_category(&db, paper_id, Some(cat_id_num)).await?;
    }

    // Copy file to attachment path
    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }
    let target_path = target_dir.join(&target_filename);

    info!("Copying PDF to: {:?}", target_path);

    std::fs::copy(&path, &target_path).map_err(|e| {
        AppError::file_system(target_path.to_string_lossy().to_string(), e.to_string())
    })?;

    // Create attachment record
    let file_size = std::fs::metadata(&target_path).ok().map(|m| m.len() as i64);

    info!("Creating attachment record");

    PaperRepository::add_attachment(
        &db,
        paper_id,
        Some(target_filename.clone()),
        Some("pdf".to_string()),
        file_size,
    )
    .await?;

    info!("PDF import completed successfully");

    let _ = app
        .notification()
        .builder()
        .title("Paper Imported from PDF")
        .body(format!("Paper '{}' imported successfully", paper.title))
        .show();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: metadata.authors,
            labels: vec![],
            attachment_count: 1,
            attachments: vec![AttachmentDto {
                id: String::new(),
                paper_id: paper_id.to_string(),
                file_name: Some(target_filename),
                file_type: Some("pdf".to_string()),
                created_at: None,
            }],
        }),
    })
}
