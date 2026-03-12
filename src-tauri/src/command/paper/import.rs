//! Import operations for papers (DOI, arXiv, PMID, PDF, Zotero RDF)

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::CreateLabel;
use crate::models::{CreateCategory, CreatePaper};
use crate::papers::importer::arxiv::{fetch_arxiv_metadata, ArxivError};
use crate::papers::importer::doi::{fetch_doi_metadata, DoiError};
use crate::papers::importer::grobid::process_header_document;
use crate::papers::importer::pubmed::{fetch_pubmed_metadata, PubmedError};
use crate::papers::importer::zotero_rdf::{parse_rdf_file, ZoteroRdfError};
use crate::repository::{AuthorRepository, CategoryRepository, LabelRepository, PaperRepository};
use crate::sys::config::AppConfig;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::calculate_attachment_hash;

/// Progress event DTO for Zotero import
#[derive(Clone, Serialize)]
pub struct ZoteroImportProgress {
    pub current: usize,
    pub total: usize,
    pub current_title: String,
    pub status: String, // "parsing", "importing", "completed", "error"
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_doi(
    _app: AppHandle,
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
        info!(
            "Paper with DOI {} already exists: {}",
            metadata.doi, existing_paper.title
        );

        return Ok(ImportResultDto {
            already_exists: true,
            message: format!(
                "Paper '{}' is already in your library",
                existing_paper.title
            ),
            paper: None,
        });
    }

    // Calculate attachment path hash
    let hash_string = calculate_attachment_hash(&metadata.title);

    // Create paper
    let publication_year = metadata
        .publication_year
        .and_then(|y| y.parse::<i32>().ok());

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
            publisher: metadata.publisher.clone(),
            issn: None,
            language: None,
        },
    )
    .await?;

    let paper_id = paper.id;

    // Add authors and create paper-author relations
    // DOI provides given/family names separately, so use create_or_find_from_parts
    for (order, author_parts) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find_from_parts(
            &db,
            author_parts.given.as_deref(),
            author_parts.family.as_deref(),
            None,
        )
        .await?;
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

    // Convert DoiAuthor to string for DTO
    let author_names: Vec<String> = metadata
        .authors
        .iter()
        .filter_map(|a| a.full_name.clone())
        .collect();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: vec![],
            attachment_count: 0,
            attachments: vec![],
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        }),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn import_paper_by_arxiv_id(
    _app: AppHandle,
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
            info!(
                "Paper with DOI {} already exists: {}",
                doi, existing_paper.title
            );

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!(
                    "Paper '{}' is already in your library",
                    existing_paper.title
                ),
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
            publisher: None,
            issn: None,
            language: None,
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
        .map_err(|e| {
            AppError::network_error(
                &metadata.pdf_url,
                format!("Failed to create HTTP client: {}", e),
            )
        })?;

    let response = client.get(&metadata.pdf_url).send().await.map_err(|e| {
        AppError::network_error(&metadata.pdf_url, format!("Failed to download PDF: {}", e))
    })?;

    if !response.status().is_success() {
        return Err(AppError::network_error(
            &metadata.pdf_url,
            format!("Failed to download PDF: HTTP {}", response.status()),
        ));
    }

    let pdf_bytes = response.bytes().await.map_err(|e| {
        AppError::network_error(
            &metadata.pdf_url,
            format!("Failed to read PDF content: {}", e),
        )
    })?;

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
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        }),
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn import_paper_by_pmid(
    _app: AppHandle,
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
            info!(
                "Paper with DOI {} already exists: {}",
                doi, existing_paper.title
            );

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!(
                    "Paper '{}' is already in your library",
                    existing_paper.title
                ),
                paper: None,
            });
        }
    }

    let pubmed_url = format!("https://pubmed.ncbi.nlm.nih.gov/{}/", metadata.pmid);
    let hash_string = calculate_attachment_hash(&metadata.title);
    let publication_year = metadata
        .publication_year
        .and_then(|y| y.parse::<i32>().ok());

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
            publisher: None,
            issn: None,
            language: None,
        },
    )
    .await?;

    let paper_id = paper.id;

    // Add authors and create paper-author relations
    // PubMed provides ForeName/LastName separately, so use create_or_find_from_parts
    for (order, author_parts) in metadata.authors.iter().enumerate() {
        let author = AuthorRepository::create_or_find_from_parts(
            &db,
            author_parts.fore_name.as_deref(),
            author_parts.last_name.as_deref(),
            None,
        )
        .await?;
        // Create paper-author relation
        PaperRepository::add_author(&db, paper_id, author.id, order as i32).await?;
    }

    if let Some(cat_id) = category_id {
        let cat_id_num = cat_id
            .parse::<i64>()
            .map_err(|_| AppError::validation("category_id", "Invalid id format"))?;
        PaperRepository::set_category(&db, paper_id, Some(cat_id_num)).await?;
    }

    // Convert PubmedAuthor to string for DTO
    let author_names: Vec<String> = metadata
        .authors
        .iter()
        .filter_map(|a| a.full_name.clone())
        .collect();

    Ok(ImportResultDto {
        already_exists: false,
        message: format!("Paper '{}' imported successfully", paper.title),
        paper: Some(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: vec![],
            attachment_count: 0,
            attachments: vec![],
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        }),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn import_paper_by_pdf(
    _app: AppHandle,
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
            info!(
                "Paper with DOI {} already exists: {}",
                doi, existing_paper.title
            );

            return Ok(ImportResultDto {
                already_exists: true,
                message: format!(
                    "Paper '{}' is already in your library",
                    existing_paper.title
                ),
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
            publication_year: metadata
                .publication_year
                .and_then(|y| i32::try_from(y).ok()),
            publication_date: None,
            journal_name: metadata.journal_name.clone(),
            conference_name: None,
            volume: None,
            issue: None,
            pages: None,
            url: None,
            abstract_text: metadata.abstract_text.clone(),
            attachment_path: Some(hash_string.clone()),
            publisher: None,
            issn: None,
            language: None,
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
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        }),
    })
}

/// Import papers from a Zotero RDF export file
///
/// This function parses a Zotero RDF file and imports all papers found in it.
/// It handles authors, attachments (PDFs), and avoids duplicates.
/// Progress events are emitted during import.
/// If no category_id is provided, a new category with name "Zotero-YYYYMMDD" is created.
#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn import_papers_from_zotero_rdf(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    file_path: String,
    category_id: Option<String>,
) -> Result<BatchImportResultDto> {
    info!("Importing papers from Zotero RDF: {}", file_path);

    // Emit initial progress
    let _ = app.emit(
        "zotero:import-progress",
        ZoteroImportProgress {
            current: 0,
            total: 0,
            current_title: String::new(),
            status: "parsing".to_string(),
        },
    );

    let rdf_path = Path::new(&file_path);
    if !rdf_path.exists() {
        let _ = app.emit(
            "zotero:import-progress",
            ZoteroImportProgress {
                current: 0,
                total: 0,
                current_title: String::new(),
                status: "error".to_string(),
            },
        );
        return Err(AppError::file_system(file_path, "RDF file not found"));
    }

    // Parse RDF file
    let items = parse_rdf_file(rdf_path).map_err(|e| {
        let _ = app.emit(
            "zotero:import-progress",
            ZoteroImportProgress {
                current: 0,
                total: 0,
                current_title: String::new(),
                status: "error".to_string(),
            },
        );
        match e {
            ZoteroRdfError::ParseError(msg) => {
                AppError::validation("rdf", format!("Failed to parse RDF file: {}", msg))
            }
            ZoteroRdfError::IoError(e) => AppError::file_system(file_path.clone(), e.to_string()),
        }
    })?;

    info!("Parsed {} items from RDF file", items.len());

    // Filter items to only include documents (not attachments or notes)
    let document_items: Vec<_> = items
        .iter()
        .filter(|item| {
            item.item_type != "attachment"
                && item.item_type != "note"
                && item.title.as_ref().is_some_and(|t| !t.is_empty())
        })
        .collect();

    let total_items = document_items.len();

    // Emit progress with total count
    let _ = app.emit(
        "zotero:import-progress",
        ZoteroImportProgress {
            current: 0,
            total: total_items,
            current_title: String::new(),
            status: "importing".to_string(),
        },
    );

    let rdf_dir = rdf_path.parent().unwrap_or(Path::new(""));

    let mut result = BatchImportResultDto {
        total: total_items,
        imported: 0,
        skipped: 0,
        failed: 0,
        papers: vec![],
        errors: vec![],
    };

    // Get or create category ID
    let cat_id_num = if let Some(ref cat_id) = category_id {
        // Use provided category ID
        Some(
            cat_id
                .parse::<i64>()
                .map_err(|_| AppError::validation("category_id", "Invalid category id format"))?,
        )
    } else {
        // Auto-create category with name "Zotero-YYYYMMDDHHMM"
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M").to_string();
        let category_name = format!("Zotero-{}", timestamp);

        info!("Auto-creating category: {}", category_name);

        let category = CategoryRepository::create(
            &db,
            CreateCategory {
                name: category_name.clone(),
                parent_id: None,
            },
        )
        .await?;

        info!(
            "Created category '{}' with id {}",
            category_name, category.id
        );
        Some(category.id)
    };

    // Process each item with progress updates
    for (index, item) in document_items.iter().enumerate() {
        let title = item.title.clone().unwrap_or_default();

        // Emit progress for current item
        let _ = app.emit(
            "zotero:import-progress",
            ZoteroImportProgress {
                current: index + 1,
                total: total_items,
                current_title: title.clone(),
                status: "importing".to_string(),
            },
        );

        // Check for duplicates by DOI
        if let Some(ref doi) = item.doi {
            if !doi.is_empty() {
                if let Some(_existing) = PaperRepository::find_by_doi(&db, doi).await? {
                    result.skipped += 1;
                    continue;
                }
            }
        }

        // Parse publication year from date
        let publication_year = item
            .date
            .as_ref()
            .and_then(|d| d.split('/').next())
            .and_then(|y| y.parse::<i32>().ok());

        // Calculate attachment hash
        let hash_string = calculate_attachment_hash(&title);

        // Create paper record
        let paper = match PaperRepository::create(
            &db,
            CreatePaper {
                title: title.clone(),
                doi: item.doi.clone().filter(|d| !d.is_empty()),
                publication_year,
                publication_date: item.date.clone(),
                journal_name: item.journal.as_ref().and_then(|j| j.title.clone()),
                conference_name: None,
                volume: item.journal.as_ref().and_then(|j| j.volume.clone()),
                issue: item.journal.as_ref().and_then(|j| j.number.clone()),
                pages: None,
                url: None,
                abstract_text: item.abstract_note.clone(),
                attachment_path: Some(hash_string.clone()),
                publisher: None,
                issn: None,
                language: None,
            },
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                result.failed += 1;
                result
                    .errors
                    .push(format!("Failed to create paper '{}': {}", title, e));
                continue;
            }
        };

        let paper_id = paper.id;

        // Add authors (with deduplication to avoid UNIQUE constraint errors)
        let mut added_author_ids: HashSet<i64> = HashSet::new();
        for (order, author) in item.authors.iter().enumerate() {
            let author_record = AuthorRepository::create_or_find_from_parts(
                &db,
                author.given_name.as_deref(),
                author.surname.as_deref(),
                None,
            )
            .await?;

            // Skip if this author was already added to this paper
            if !added_author_ids.insert(author_record.id) {
                continue;
            }

            PaperRepository::add_author(&db, paper_id, author_record.id, order as i32).await?;
        }

        // Add tags (labels) with deduplication
        let mut added_tag_names: HashSet<&str> = HashSet::new();
        for tag_name in &item.tags {
            let tag_name = tag_name.trim();
            if tag_name.is_empty() {
                continue;
            }

            // Skip if this tag was already processed for this paper
            if !added_tag_names.insert(tag_name) {
                continue;
            }

            // Find or create label
            let label = if let Some(existing) = LabelRepository::find_by_name(&db, tag_name).await?
            {
                existing
            } else {
                LabelRepository::create(
                    &db,
                    CreateLabel {
                        name: tag_name.to_string(),
                        color: "#607D8B".to_string(), // Default gray color
                    },
                )
                .await?
            };

            // Add label to paper (ignore if already exists)
            if let Err(e) = LabelRepository::add_to_paper(&db, paper_id, label.id).await {
                // Log but don't fail if the label is already associated with this paper
                info!("Label '{}' already associated with paper: {}", tag_name, e);
            }
        }

        // Set category
        if let Some(cat_id) = cat_id_num {
            PaperRepository::set_category(&db, paper_id, Some(cat_id)).await?;
        }

        // Process attachments (PDFs)
        let mut attachment_count = 0;
        let mut attachments_dto: Vec<AttachmentDto> = vec![];

        info!(
            "Processing {} attachments for paper: {}",
            item.attachments.len(),
            paper.title
        );

        for attachment in &item.attachments {
            info!(
                "Attachment: title={:?}, path={:?}, content_type={:?}",
                attachment.title, attachment.path, attachment.content_type
            );

            // Resolve attachment path relative to RDF file
            let attachment_path_str = match &attachment.path {
                Some(path) => path,
                None => {
                    info!("Attachment has no local path, skipping");
                    continue;
                }
            };

            let attachment_path = rdf_dir.join(attachment_path_str);
            info!("Resolved attachment path: {:?}", attachment_path);

            if !attachment_path.exists() {
                info!("Attachment file not found: {:?}", attachment_path);
                continue;
            }

            // Create target directory
            let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
            if !target_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&target_dir) {
                    result
                        .errors
                        .push(format!("Failed to create attachment directory: {}", e));
                    continue;
                }
            }

            // Get filename from attachment title or path
            let filename = attachment.title.clone().unwrap_or_else(|| {
                attachment_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "attachment.pdf".to_string())
            });

            let target_path = target_dir.join(&filename);

            // Copy attachment file
            if let Err(e) = std::fs::copy(&attachment_path, &target_path) {
                result
                    .errors
                    .push(format!("Failed to copy attachment '{}': {}", filename, e));
                continue;
            }

            // Create attachment record
            let file_size = std::fs::metadata(&target_path).ok().map(|m| m.len() as i64);

            if let Err(e) = PaperRepository::add_attachment(
                &db,
                paper_id,
                Some(filename.clone()),
                Some("pdf".to_string()),
                file_size,
            )
            .await
            {
                result
                    .errors
                    .push(format!("Failed to create attachment record: {}", e));
                continue;
            }

            attachment_count += 1;
            attachments_dto.push(AttachmentDto {
                id: String::new(),
                paper_id: paper_id.to_string(),
                file_name: Some(filename),
                file_type: Some("pdf".to_string()),
                created_at: None,
            });
        }

        // Build author names for DTO
        let author_names: Vec<String> = item.authors.iter().map(|a| a.display_name()).collect();

        result.imported += 1;
        result.papers.push(PaperDto {
            id: paper_id.to_string(),
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: author_names,
            labels: vec![],
            attachment_count,
            attachments: attachments_dto,
            publisher: paper.publisher,
            issn: paper.issn,
            language: paper.language,
        });
    }

    // Emit completion progress
    let _ = app.emit(
        "zotero:import-progress",
        ZoteroImportProgress {
            current: total_items,
            total: total_items,
            current_title: String::new(),
            status: "completed".to_string(),
        },
    );

    info!(
        "Zotero RDF import completed: {} imported, {} skipped, {} failed",
        result.imported, result.skipped, result.failed
    );

    // Emit paper:imported event to refresh paper list
    let _ = app.emit(
        "paper:imported",
        serde_json::json!({
            "imported": result.imported,
            "skipped": result.skipped,
            "failed": result.failed
        }),
    );

    // Emit category:refresh event to refresh category tree
    let _ = app.emit("category:refresh", ());

    Ok(result)
}
