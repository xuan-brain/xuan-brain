//! Zotero RDF import module
//!
//! This module provides functionality to parse Zotero RDF export files
//! and extract paper metadata including authors, attachments, etc.

use std::path::Path;

use thiserror::Error;
use zotero_rdf::{parse_file, Extractor, ZoteroItem};

/// Zotero RDF import error types
#[derive(Error, Debug)]
pub enum ZoteroRdfError {
    #[error("Failed to parse RDF file: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Parse a Zotero RDF file and extract all items
///
/// # Arguments
/// * `rdf_path` - Path to the RDF file
///
/// # Returns
/// A vector of ZoteroItem containing parsed metadata
pub fn parse_rdf_file(rdf_path: &Path) -> Result<Vec<ZoteroItem>, ZoteroRdfError> {
    let graph =
        parse_file(rdf_path).map_err(|e| ZoteroRdfError::ParseError(e.to_string()))?;
    let extractor = Extractor::new(&graph);
    let items = extractor.extract_all();
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rdf_file() {
        // This test requires a real RDF file
        // Run manually with a valid RDF file path
        let rdf_path = Path::new("C:/Users/guo/Downloads/我的文库/我的文库.rdf");
        if rdf_path.exists() {
            let result = parse_rdf_file(rdf_path);
            match result {
                Ok(items) => {
                    println!("Parsed {} items", items.len());
                    for item in items.iter().take(5) {
                        println!("Item: {} - {:?}", item.item_type, item.title);
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
}
