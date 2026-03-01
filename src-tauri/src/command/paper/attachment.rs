//! Attachment operations for papers

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;
use tracing::{info, instrument};

use crate::database::DatabaseConnection;
use crate::models::Attachment;
use crate::repository::PaperRepository;
use crate::sys::dirs::AppDirs;
use crate::sys::error::{AppError, Result};

use super::dtos::*;
use super::utils::{base64_decode, base64_encode, calculate_attachment_hash};
use chrono::Utc;

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn add_attachment(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: String,
    file_path: String,
) -> Result<AttachmentDto> {
    info!("Adding attachment for paper {}: {}", paper_id, file_path);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }

    let source_path = PathBuf::from(&file_path);
    let file_name = source_path.file_name()
        .ok_or_else(|| AppError::validation("file_path", "Invalid file path"))?
        .to_string_lossy().to_string();
    let target_path = target_dir.join(&file_name);

    std::fs::copy(&source_path, &target_path).map_err(|e| {
        AppError::file_system(target_path.to_string_lossy().to_string(), e.to_string())
    })?;

    let file_type = source_path.extension().map(|s| s.to_string_lossy().to_string());
    let file_size = std::fs::metadata(&target_path).ok().map(|m| m.len() as i64);

    let attachment = Attachment {
        id: 0, // Will be auto-generated
        paper_id: paper_id_num,
        file_name: Some(file_name.clone()),
        file_type: file_type.clone(),
        file_size,
        created_at: Utc::now(),
    };

    PaperRepository::add_attachment_model(&db, attachment).await?;

    let _ = app
        .notification()
        .builder()
        .title("Attachment Added")
        .body("Attachment added successfully")
        .show();

    Ok(AttachmentDto {
        id: String::new(),
        paper_id: paper_id.clone(),
        file_name: Some(file_name),
        file_type,
        created_at: Some(Utc::now().to_rfc3339()),
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn get_attachments(
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: String,
) -> Result<Vec<AttachmentDto>> {
    info!("Fetching attachments for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let attachments = PaperRepository::get_attachments(&db, paper_id_num).await?;

    Ok(attachments.iter().map(|a| AttachmentDto {
        id: a.id.to_string(),
        paper_id: a.paper_id.to_string(),
        file_name: a.file_name.clone(),
        file_type: a.file_type.clone(),
        created_at: Some(a.created_at.to_rfc3339()),
    }).collect())
}

#[tauri::command]
#[instrument(skip(db, app_dirs, app))]
pub async fn open_paper_folder(
    app: AppHandle,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: String,
) -> Result<()> {
    info!("Opening folder for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let target_dir = PathBuf::from(&app_dirs.files).join(&hash_string);

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir).map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;
    }

    app.opener()
        .open_path(target_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| {
            AppError::file_system(target_dir.to_string_lossy().to_string(), e.to_string())
        })?;

    Ok(())
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn get_pdf_attachment_path(
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
    paper_id: String,
) -> Result<PdfAttachmentInfo> {
    info!("Getting PDF attachment path for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let attachment = PaperRepository::find_pdf_attachment(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!("{}.pdf", paper.title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_"))
    });

    let files_dir = PathBuf::from(&app_dirs.files);
    let pdf_path = files_dir.join(&hash_string).join(&file_name);

    if !pdf_path.exists() {
        return Err(AppError::not_found("PDF file", format!("hash={}", hash_string)));
    }

    Ok(PdfAttachmentInfo {
        file_path: pdf_path.to_string_lossy().to_string(),
        file_name,
        paper_id,
        paper_title: paper.title,
        base64_content: None,
    })
}

#[tauri::command]
#[instrument(skip(app_dirs))]
pub async fn read_pdf_file(app_dirs: State<'_, AppDirs>, file_path: String) -> Result<Vec<u8>> {
    info!("Reading PDF file: {}", file_path);

    let path = PathBuf::from(&file_path);
    let files_dir = PathBuf::from(&app_dirs.files);

    if !path.starts_with(&files_dir) {
        return Err(AppError::permission(format!(
            "file_read: Path {} is not within the allowed directory",
            file_path
        )));
    }

    let contents = std::fs::read(&path).map_err(|e| {
        AppError::file_system(file_path.clone(), format!("Failed to read file: {}", e))
    })?;

    info!("Successfully read PDF file, size: {} bytes", contents.len());
    Ok(contents)
}

#[tauri::command]
#[instrument(skip(db, app_dirs))]
pub async fn read_pdf_as_blob(
    paper_id: String,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfBlobResponse> {
    info!("Reading PDF as blob for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let attachment = PaperRepository::find_pdf_attachment(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!("{}.pdf", paper.title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_"))
    });

    let files_dir = PathBuf::from(&app_dirs.files);
    let pdf_path = files_dir.join(&hash_string).join(&file_name);

    if !pdf_path.exists() {
        return Err(AppError::not_found("PDF file", format!("hash={}", hash_string)));
    }

    let pdf_bytes = std::fs::read(&pdf_path).map_err(|e| {
        AppError::file_system(pdf_path.to_string_lossy().to_string(), format!("Failed to read PDF file: {}", e))
    })?;

    let size_bytes = pdf_bytes.len();
    let base64_data = base64_encode(&pdf_bytes);

    info!("Successfully read PDF as blob for paper {}: {} bytes", paper_id, size_bytes);

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
    paper_id: String,
    base64_data: String,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfSaveResponse> {
    info!("Saving PDF blob for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let attachment = PaperRepository::find_pdf_attachment(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!("{}.pdf", paper.title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_"))
    });

    let pdf_bytes = base64_decode(&base64_data).map_err(|e| {
        AppError::validation("base64_data", format!("Failed to decode base64: {}", e))
    })?;

    let size_bytes = pdf_bytes.len();

    let files_dir = PathBuf::from(&app_dirs.files);
    let pdf_path = files_dir.join(&hash_string).join(&file_name);

    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(parent.to_string_lossy().to_string(), e.to_string())
        })?;
    }

    std::fs::write(&pdf_path, &pdf_bytes).map_err(|e| {
        AppError::file_system(pdf_path.to_string_lossy().to_string(), e.to_string())
    })?;

    info!("Successfully saved PDF blob for paper {}: {} bytes", paper_id, size_bytes);

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
        message: format!("PDF saved successfully: {} ({} bytes)", file_name, size_bytes),
    })
}

#[tauri::command]
#[instrument(skip(db, app_dirs, base64_data, app))]
pub async fn save_pdf_with_annotations(
    app: AppHandle,
    paper_id: String,
    base64_data: String,
    annotations_json: Option<String>,
    db: State<'_, Arc<DatabaseConnection>>,
    app_dirs: State<'_, AppDirs>,
) -> Result<PdfSaveResponse> {
    info!("Saving PDF blob with annotations for paper {}", paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    let paper = PaperRepository::find_by_id(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("Paper", paper_id.clone()))?;

    let hash_string = paper.attachment_path.clone().unwrap_or_else(|| {
        calculate_attachment_hash(&paper.title)
    });

    let attachment = PaperRepository::find_pdf_attachment(&db, paper_id_num).await?
        .ok_or_else(|| AppError::not_found("PDF attachment", format!("paper_id={}", paper_id)))?;

    let file_name = attachment.file_name.clone().unwrap_or_else(|| {
        format!("{}.pdf", paper.title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "_"))
    });

    let pdf_bytes = base64_decode(&base64_data).map_err(|e| {
        AppError::validation("base64_data", format!("Failed to decode base64: {}", e))
    })?;

    let size_bytes = pdf_bytes.len();

    let files_dir = PathBuf::from(&app_dirs.files);
    let pdf_path = files_dir.join(&hash_string).join(&file_name);

    if let Some(parent) = pdf_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::file_system(parent.to_string_lossy().to_string(), e.to_string())
        })?;
    }

    std::fs::write(&pdf_path, &pdf_bytes).map_err(|e| {
        AppError::file_system(pdf_path.to_string_lossy().to_string(), e.to_string())
    })?;

    if let Some(annotations) = annotations_json {
        let annotations_path = pdf_path.with_extension("json");
        std::fs::write(&annotations_path, &annotations).map_err(|e| {
            AppError::file_system(annotations_path.to_string_lossy().to_string(), e.to_string())
        })?;

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
            message: format!("PDF and annotations saved successfully ({} bytes)", size_bytes),
        });
    }

    Ok(PdfSaveResponse {
        success: true,
        file_path: pdf_path.to_string_lossy().to_string(),
        size_bytes,
        message: format!("PDF saved successfully: {} ({} bytes)", file_name, size_bytes),
    })
}

#[tauri::command]
#[instrument(skip(db))]
pub async fn delete_attachment(
    db: State<'_, Arc<DatabaseConnection>>,
    paper_id: String,
    file_name: String,
) -> Result<()> {
    info!("Deleting attachment {} for paper {}", file_name, paper_id);

    let paper_id_num = paper_id.parse::<i64>()
        .map_err(|_| AppError::validation("paper_id", "Invalid paper id format"))?;

    PaperRepository::remove_attachment_by_name(&db, paper_id_num, &file_name).await?;

    info!("Successfully deleted attachment {} for paper {}", file_name, paper_id);
    Ok(())
}
