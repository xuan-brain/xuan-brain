//! Relationship models for SurrealDB graph edges

use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

/// Paper-Author relationship (edge)
/// Represents the relationship between a paper and an author
/// Note: `in` and `out` are Rust reserved keywords, so we use `in_paper`/`out_author` naming
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaperAuthor {
    pub id: Option<String>,
    /// Paper reference (the "in" side of the relation)
    #[serde(rename = "in")]
    pub in_paper: String,
    /// Author reference (the "out" side of the relation)
    #[serde(rename = "out")]
    pub out_author: String,
    pub author_order: i32,
    pub is_corresponding: bool,
}

/// Paper-Keyword relationship (edge)
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaperKeyword {
    pub id: Option<String>,
    #[serde(rename = "in")]
    pub in_paper: String,
    #[serde(rename = "out")]
    pub out_keyword: String,
}

/// Paper-Label relationship (edge)
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaperLabel {
    pub id: Option<String>,
    #[serde(rename = "in")]
    pub in_paper: String,
    #[serde(rename = "out")]
    pub out_label: String,
}

/// Paper-Category relationship (edge)
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct PaperCategory {
    pub id: Option<String>,
    #[serde(rename = "in")]
    pub in_paper: String,
    #[serde(rename = "out")]
    pub out_category: String,
}
