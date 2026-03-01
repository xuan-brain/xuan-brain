//! Utility functions for paper commands

use sha1::{Digest, Sha1};

/// Calculate SHA1 hash of title for attachment path
pub fn calculate_attachment_hash(title: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(title.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Base64 encoding
pub fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.encode(data)
}

/// Base64 decoding
pub fn base64_decode(data: &str) -> std::result::Result<Vec<u8>, String> {
    use base64::{Engine as _, engine::general_purpose};
    general_purpose::STANDARD.decode(data).map_err(|e| e.to_string())
}

/// Parse string ID to i64
pub fn parse_id(id: &str) -> Result<i64, String> {
    id.parse::<i64>().map_err(|_| format!("Invalid id format: {}", id))
}
