use crate::sys::error::{AppError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::multipart;
use std::path::Path;
use tokio::fs;
use tracing::info;

#[derive(Debug, Default)]
pub struct GrobidMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub doi: Option<String>,
    pub abstract_text: Option<String>,
    pub publication_year: Option<i64>,
    pub journal_name: Option<String>,
}

pub async fn process_header_document(file_path: &Path, server_url: &str) -> Result<GrobidMetadata> {
    // 1. Read file
    let file_bytes = fs::read(file_path).await?;
    let file_part = multipart::Part::bytes(file_bytes)
        .file_name(
            file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        )
        .mime_str("application/pdf")
        .map_err(|e| {
            AppError::network_error(server_url, format!("Failed to create multipart: {}", e))
        })?;

    let form = multipart::Form::new().part("input", file_part);

    // 2. Send request
    let client = reqwest::Client::builder().no_proxy().build().map_err(|e| {
        AppError::network_error(server_url, format!("Failed to create client: {}", e))
    })?;

    let url = format!(
        "{}/api/processHeaderDocument",
        server_url.trim_end_matches('/')
    );

    let response = client
        .post(&url)
        .header("Accept", "application/xml")
        .multipart(form)
        .send()
        .await
        .map_err(|e| AppError::network_error(&url, format!("GROBID request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::network_error(
            &url,
            format!("GROBID returned status: {}", response.status()),
        ));
    }

    let xml_content = response.text().await.map_err(|e| {
        AppError::network_error(&url, format!("Failed to read GROBID response: {}", e))
    })?;

    info!("\n========== GROBID RAW XML RESPONSE ==========");
    info!("{}", xml_content);
    info!("========== END GROBID RESPONSE ==========\n");

    // 3. Parse XML
    parse_tei_xml(&xml_content)
}

#[allow(unused_assignments, unused_variables)]
fn parse_tei_xml(xml: &str) -> Result<GrobidMetadata> {
    info!("Attempting to parse TEI XML response");
    info!(
        "Raw XML content (first 500 chars): {}...",
        xml.chars().take(500).collect::<String>()
    );

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut metadata = GrobidMetadata::default();
    let mut buf = Vec::new();

    // State flags
    let mut in_title_stmt = false;
    let mut in_analytic = false;
    let mut in_monogr = false;
    let mut in_author = false;
    let mut in_surname = false;
    let mut in_forename = false;
    let mut in_abstract = false;
    let mut current_author = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"titleStmt" => in_title_stmt = true,
                b"analytic" => in_analytic = true,
                b"monogr" => in_monogr = true,
                b"author" => {
                    in_author = true;
                    current_author.clear();
                    info!("Starting to parse author");
                }
                b"surname" => in_surname = true,
                b"forename" => in_forename = true,
                b"abstract" => in_abstract = true,
                b"title" => {
                    if in_analytic {
                        if let Ok(title) = reader.read_text(e.name()) {
                            if !title.trim().is_empty() {
                                metadata.title = title.to_string();
                                info!("Extracted title from analytic: {}", metadata.title);
                            }
                        }
                    } else if in_title_stmt && metadata.title.is_empty() {
                        if let Ok(title) = reader.read_text(e.name()) {
                            metadata.title = title.to_string();
                            info!("Extracted title from titleStmt: {}", metadata.title);
                        }
                    } else if in_monogr {
                        if let Ok(journal) = reader.read_text(e.name()) {
                            metadata.journal_name = Some(journal.to_string());
                            info!(
                                "Extracted journal name: {}",
                                metadata.journal_name.as_ref().unwrap()
                            );
                        }
                    }
                }
                b"idno" => {
                    let mut is_doi = false;
                    for a in e.attributes().flatten() {
                        if a.key.as_ref() == b"type" && a.value.as_ref() == b"DOI" {
                            is_doi = true;
                            break;
                        }
                    }
                    if is_doi {
                        if let Ok(doi) = reader.read_text(e.name()) {
                            metadata.doi = Some(doi.to_string());
                            info!("Extracted DOI: {}", metadata.doi.as_ref().unwrap());
                        }
                    }
                }
                b"date" => {
                    if in_monogr {
                        e.attributes().for_each(|attr| {
                            if let Ok(a) = attr {
                                if a.key.as_ref() == b"when" {
                                    let date_str = String::from_utf8_lossy(a.value.as_ref());
                                    if let Some(year) = date_str.split('-').next() {
                                        metadata.publication_year = year.parse().ok();
                                        if let Some(y) = metadata.publication_year {
                                            info!("Extracted publication year: {}", y);
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"titleStmt" => in_title_stmt = false,
                b"analytic" => in_analytic = false,
                b"monogr" => in_monogr = false,
                b"author" => {
                    in_author = false;
                    let name = current_author.trim();
                    if !name.is_empty() {
                        metadata.authors.push(name.to_string());
                        info!("Added author: {}", name);
                    }
                }
                b"surname" => in_surname = false,
                b"forename" => in_forename = false,
                b"abstract" => in_abstract = false,
                _ => (),
            },
            Ok(Event::Text(e)) => {
                let text = String::from_utf8_lossy(&e).to_string();
                if in_surname || in_forename {
                    current_author.push_str(&text);
                    current_author.push(' ');
                } else if in_abstract {
                    if let Some(abs) = &mut metadata.abstract_text {
                        abs.push_str(&text);
                        abs.push(' ');
                    } else {
                        metadata.abstract_text = Some(text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => (),
        }
        buf.clear();
    }

    info!("Parsing completed. Final metadata: {:?}", metadata);
    info!(
        "Title: {}, Authors: {}, DOI: {:?}, Year: {:?}, Journal: {:?}, Abstract length: {}",
        metadata.title,
        metadata.authors.len(),
        metadata.doi,
        metadata.publication_year,
        metadata.journal_name,
        metadata.abstract_text.as_ref().map_or(0, |s| s.len())
    );

    Ok(metadata)
}
