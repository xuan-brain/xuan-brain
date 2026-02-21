//! Utility functions for paper commands

use sha1::{Digest, Sha1};
use surrealdb_types::RecordIdKey;

/// Convert RecordId to string
pub fn record_id_to_string(id: &surrealdb_types::RecordId) -> String {
    format!("{}:{}", id.table, record_id_key_to_string(&id.key))
}

fn record_id_key_to_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        RecordIdKey::Array(_) => "array".to_string(),
        RecordIdKey::Object(_) => "object".to_string(),
        RecordIdKey::Range(_) => "range".to_string(),
    }
}

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
