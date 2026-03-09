//! Author domain model

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::entities::author;

/// Author record representing a research paper author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub affiliation: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// DTO for creating a new author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuthor {
    pub first_name: String,
    pub last_name: Option<String>,
    pub affiliation: Option<String>,
    pub email: Option<String>,
}

/// Structured author name parts for importers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorNameParts {
    pub first_name: String,
    pub last_name: Option<String>,
}

/// Helper struct for parsing author names from various sources
#[derive(Debug, Clone)]
pub struct AuthorNameParser;

impl Author {
    /// Get full name by combining first_name and last_name
    pub fn full_name(&self) -> String {
        match &self.last_name {
            Some(last) if !last.is_empty() => format!("{} {}", self.first_name, last),
            _ => self.first_name.clone(),
        }
    }
}

impl AuthorNameParser {
    /// Parse a full name string into first_name and last_name
    ///
    /// # Name Format Support
    ///
    /// - Western: "John Smith" -> first_name: "John", last_name: "Smith"
    /// - Western with middle: "John Robert Smith" -> first_name: "John Robert", last_name: "Smith"
    /// - Chinese: "张三" -> first_name: "张三", last_name: None (full name in first_name)
    /// - Single name: "Plato" -> first_name: "Plato", last_name: None
    /// - Citation format: "Smith, John" -> first_name: "John", last_name: "Smith"
    pub fn parse(full_name: &str) -> AuthorNameParts {
        let name = full_name.trim();

        if name.is_empty() {
            return AuthorNameParts {
                first_name: String::new(),
                last_name: None,
            };
        }

        // Check for "Last, First" format (common in citations)
        if let Some((last, first)) = name.split_once(',') {
            return AuthorNameParts {
                first_name: first.trim().to_string(),
                last_name: Some(last.trim().to_string()),
            };
        }

        // Split by spaces
        let parts: Vec<&str> = name.split_whitespace().collect();

        match parts.len() {
            0 => AuthorNameParts {
                first_name: String::new(),
                last_name: None,
            },
            1 => AuthorNameParts {
                // Single name - could be Chinese, mononym, etc.
                first_name: parts[0].to_string(),
                last_name: None,
            },
            2 => AuthorNameParts {
                // Standard "First Last" format
                first_name: parts[0].to_string(),
                last_name: Some(parts[1].to_string()),
            },
            _ => {
                // Multiple parts: "First Middle Last" or "First Middle1 Middle2 Last"
                // Convention: last word is last_name, rest is first_name
                let last_idx = parts.len() - 1;
                AuthorNameParts {
                    first_name: parts[..last_idx].join(" "),
                    last_name: Some(parts[last_idx].to_string()),
                }
            }
        }
    }

    /// Parse from given name and family name (already split by source)
    /// This is used for DOI (given/family) and PubMed (ForeName/LastName)
    pub fn from_parts(given: Option<&str>, family: Option<&str>) -> AuthorNameParts {
        match (given, family) {
            (Some(g), Some(f)) if !g.trim().is_empty() && !f.trim().is_empty() => AuthorNameParts {
                first_name: g.trim().to_string(),
                last_name: Some(f.trim().to_string()),
            },
            (Some(g), Some(_)) if !g.trim().is_empty() => AuthorNameParts {
                // family is empty, use only given
                first_name: g.trim().to_string(),
                last_name: None,
            },
            (Some(g), None) if !g.trim().is_empty() => Self::parse(g),
            (None, Some(f)) if !f.trim().is_empty() => AuthorNameParts {
                // only family name available
                first_name: f.trim().to_string(),
                last_name: None,
            },
            _ => AuthorNameParts {
                first_name: String::new(),
                last_name: None,
            },
        }
    }
}

impl From<CreateAuthor> for Author {
    fn from(create: CreateAuthor) -> Self {
        Self {
            id: 0,
            first_name: create.first_name,
            last_name: create.last_name,
            affiliation: create.affiliation,
            email: create.email,
            created_at: Utc::now(),
        }
    }
}

impl From<author::Model> for Author {
    fn from(model: author::Model) -> Self {
        Self {
            id: model.id,
            first_name: model.first_name,
            last_name: model.last_name,
            affiliation: model.affiliation,
            email: model.email,
            created_at: model.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_western_name() {
        let name = AuthorNameParser::parse("John Smith");
        assert_eq!(name.first_name, "John");
        assert_eq!(name.last_name, Some("Smith".to_string()));
    }

    #[test]
    fn test_parse_name_with_middle() {
        let name = AuthorNameParser::parse("John Robert Smith");
        assert_eq!(name.first_name, "John Robert");
        assert_eq!(name.last_name, Some("Smith".to_string()));
    }

    #[test]
    fn test_parse_single_name() {
        let name = AuthorNameParser::parse("Plato");
        assert_eq!(name.first_name, "Plato");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_parse_chinese_name() {
        // Chinese names are typically 2-3 characters without spaces
        let name = AuthorNameParser::parse("张三");
        assert_eq!(name.first_name, "张三");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_parse_last_first_format() {
        let name = AuthorNameParser::parse("Smith, John");
        assert_eq!(name.first_name, "John");
        assert_eq!(name.last_name, Some("Smith".to_string()));
    }

    #[test]
    fn test_parse_multiple_middle_names() {
        let name = AuthorNameParser::parse("John Robert William Smith");
        assert_eq!(name.first_name, "John Robert William");
        assert_eq!(name.last_name, Some("Smith".to_string()));
    }

    #[test]
    fn test_parse_empty_name() {
        let name = AuthorNameParser::parse("");
        assert_eq!(name.first_name, "");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let name = AuthorNameParser::parse("   ");
        assert_eq!(name.first_name, "");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_from_parts_both() {
        let name = AuthorNameParser::from_parts(Some("John"), Some("Smith"));
        assert_eq!(name.first_name, "John");
        assert_eq!(name.last_name, Some("Smith".to_string()));
    }

    #[test]
    fn test_from_parts_given_only() {
        let name = AuthorNameParser::from_parts(Some("张三"), None);
        assert_eq!(name.first_name, "张三");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_from_parts_family_only() {
        let name = AuthorNameParser::from_parts(None, Some("Plato"));
        assert_eq!(name.first_name, "Plato");
        assert_eq!(name.last_name, None);
    }

    #[test]
    fn test_author_full_name() {
        let author = Author {
            id: 1,
            first_name: "John".to_string(),
            last_name: Some("Smith".to_string()),
            affiliation: None,
            email: None,
            created_at: Utc::now(),
        };
        assert_eq!(author.full_name(), "John Smith");
    }

    #[test]
    fn test_author_full_name_no_last() {
        let author = Author {
            id: 1,
            first_name: "张三".to_string(),
            last_name: None,
            affiliation: None,
            email: None,
            created_at: Utc::now(),
        };
        assert_eq!(author.full_name(), "张三");
    }
}
