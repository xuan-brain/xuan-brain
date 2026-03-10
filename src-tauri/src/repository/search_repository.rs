//! Search repository using SQLite FTS5 full-text search
//!
//! Provides efficient full-text search across paper title, abstract,
//! labels, and attachments using SQLite's FTS5 extension with BM25 scoring.

use chrono::{DateTime, Utc};
use sea_orm::{ConnectionTrait, DbBackend, *};
use tracing::info;

use crate::database::entities::paper;
use crate::sys::error::{AppError, Result};

// Import sqlx types from SeaORM's re-export
use sea_orm::sqlx::{Row, sqlite::SqliteRow};

/// Repository for full-text search operations
pub struct SearchRepository;

impl SearchRepository {
    /// Full-text search using FTS5 with BM25 relevance scoring
    ///
    /// Returns papers with their relevance scores (0-100, higher is better)
    ///
    /// # Arguments
    /// * `db` - Database connection
    /// * `query` - Search query string (supports FTS5 query syntax)
    /// * `limit` - Maximum number of results to return (default: 50)
    pub async fn fts_search(
        db: &DatabaseConnection,
        query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<(paper::Model, f64)>> {
        let limit = limit.unwrap_or(50);

        info!("FTS search query: '{}'", query);

        // Sanitize query by removing potential SQL injection
        let sanitized_query = query.replace('\\', "\\\\").replace('"', "\\\"");

        // For Chinese text, we need to handle it differently
        // unicode61 tokenizer treats each Chinese character as a separate token
        // Chinese characters should NOT be quoted in FTS5 queries
        let fts_query = if Self::contains_chinese(&sanitized_query) {
            // Split Chinese characters and join with OR for better matching
            let chars: Vec<String> = sanitized_query
                .chars()
                .filter(|c| !c.is_whitespace())
                .map(|c| c.to_string())
                .collect();
            if chars.len() > 1 {
                // FTS5 OR query syntax for Chinese: term1 OR term2 OR term3
                // No quotes around Chinese characters
                chars.join(" OR ")
            } else {
                // Single Chinese character - no quotes
                sanitized_query.clone()
            }
        } else {
            // For non-Chinese, use the original query (phrase search)
            format!("\"{}\"", sanitized_query)
        };

        info!("FTS search processed query: '{}'", fts_query);

        // Build FTS5 query with BM25 scoring
        // Use subquery approach for better FTS5 external content support
        let sql = format!(
            r#"
            SELECT
                p.id, p.title, p.abstract_text, p.doi, p.publication_year,
                p.publication_date, p.journal_name, p.conference_name, p.volume,
                p.issue, p.pages, p.url, p.citation_count, p.read_status,
                p.notes, p.attachment_path, p.created_at, p.updated_at,
                p.deleted_at, p.publisher, p.issn, p.language, p.attachment_count,
                fts.score
            FROM paper p
            INNER JOIN (
                SELECT paper_id, -bm25(paper_fts) AS score
                FROM paper_fts
                WHERE paper_fts MATCH '{}'
            ) fts ON p.id = fts.paper_id
            WHERE p.deleted_at IS NULL
            ORDER BY fts.score DESC
            LIMIT {}
            "#,
            fts_query, limit
        );

        // Execute query using sqlx directly through SeaORM's connection
        let sqlx_rows: Vec<SqliteRow> = match db.get_database_backend() {
            DbBackend::Sqlite => {
                let pool = db.get_sqlite_connection_pool();
                sqlx::query(&sql)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| AppError::generic(format!("Failed to execute FTS search: {}", e)))?
            }
            _ => {
                return Err(AppError::generic(
                    "FTS search is only supported for SQLite databases".to_string(),
                ))
            }
        };

        let mut search_results = Vec::new();

        for row in sqlx_rows {
            // Extract all fields from the row manually by column index
            // Column indices: 0=id, 1=title, 2=abstract_text, 3=doi, 4=publication_year,
            // 5=publication_date, 6=journal_name, 7=conference_name, 8=volume,
            // 9=issue, 10=pages, 11=url, 12=citation_count, 13=read_status,
            // 14=notes, 15=attachment_path, 16=created_at, 17=updated_at,
            // 18=deleted_at, 19=publisher, 20=issn, 21=language, 22=attachment_count,
            // 23=score

            let paper_id: i64 = row
                .try_get::<i64, _>(0)
                .map_err(|e| AppError::generic(format!("Failed to get id: {}", e)))?;
            let title: String = row
                .try_get::<String, _>(1)
                .map_err(|e| AppError::generic(format!("Failed to get title: {}", e)))?;

            // Try get optional fields, using ok() to handle None values
            let abstract_text: Option<String> = row.try_get::<Option<String>, _>(2).ok().flatten();
            let doi: Option<String> = row.try_get::<Option<String>, _>(3).ok().flatten();
            let publication_year: Option<i32> = row.try_get::<Option<i32>, _>(4).ok().flatten();
            let publication_date: Option<String> = row.try_get::<Option<String>, _>(5).ok().flatten();
            let journal_name: Option<String> = row.try_get::<Option<String>, _>(6).ok().flatten();
            let conference_name: Option<String> = row.try_get::<Option<String>, _>(7).ok().flatten();
            let volume: Option<String> = row.try_get::<Option<String>, _>(8).ok().flatten();
            let issue: Option<String> = row.try_get::<Option<String>, _>(9).ok().flatten();
            let pages: Option<String> = row.try_get::<Option<String>, _>(10).ok().flatten();
            let url: Option<String> = row.try_get::<Option<String>, _>(11).ok().flatten();
            let citation_count: i32 = row.try_get::<Option<i32>, _>(12).ok().flatten().unwrap_or(0);
            let read_status: String = row
                .try_get::<Option<String>, _>(13)
                .ok()
                .flatten()
                .unwrap_or("unread".to_string());
            let notes: Option<String> = row.try_get::<Option<String>, _>(14).ok().flatten();
            let attachment_path: Option<String> = row.try_get::<Option<String>, _>(15).ok().flatten();

            // Parse datetime strings to DateTime<Utc> (indices 16=created_at, 17=updated_at)
            let created_at_str: String = row
                .try_get::<String, _>(16)
                .map_err(|e| AppError::generic(format!("Failed to get created_at: {}", e)))?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let updated_at_str: String = row
                .try_get::<String, _>(17)
                .map_err(|e| AppError::generic(format!("Failed to get updated_at: {}", e)))?;
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            // Handle deleted_at (index 18)
            let deleted_at: Option<DateTime<Utc>> = {
                match row.try_get::<Option<String>, _>(18) {
                    Ok(Some(s)) => DateTime::parse_from_rfc3339(&s)
                        .map(|dt| Some(dt.with_timezone(&Utc)))
                        .unwrap_or(None),
                    Ok(None) => None,
                    Err(_) => None,
                }
            };

            let publisher: Option<String> = row.try_get::<Option<String>, _>(19).ok().flatten();
            let issn: Option<String> = row.try_get::<Option<String>, _>(20).ok().flatten();
            let language: Option<String> = row.try_get::<Option<String>, _>(21).ok().flatten();
            let attachment_count: i32 = row.try_get::<Option<i32>, _>(22).ok().flatten().unwrap_or(0);

            // Get score (last column, index 23)
            let raw_score: f64 = row.try_get::<Option<f64>, _>(23).ok().flatten().unwrap_or(0.0);

            // Normalize score to 0-100 range
            let normalized_score = Self::normalize_score(raw_score);

            search_results.push((
                paper::Model {
                    id: paper_id,
                    title,
                    abstract_text,
                    doi,
                    publication_year,
                    publication_date,
                    journal_name,
                    conference_name,
                    volume,
                    issue,
                    pages,
                    url,
                    citation_count,
                    read_status,
                    notes,
                    attachment_path,
                    created_at,
                    updated_at,
                    deleted_at,
                    publisher,
                    issn,
                    language,
                    attachment_count,
                },
                normalized_score,
            ));
        }

        info!(
            "FTS search for '{}' found {} results",
            query,
            search_results.len()
        );
        Ok(search_results)
    }

    /// Get search suggestions based on prefix matching
    ///
    /// Returns paper titles that start with the given prefix
    pub async fn get_search_suggestions(
        db: &DatabaseConnection,
        prefix: &str,
        limit: u64,
    ) -> Result<Vec<String>> {
        let limit = std::cmp::Ord::min(limit, 20); // Cap at 20 suggestions

        // Escape wildcard characters
        let escaped_prefix = prefix.replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("{}%", escaped_prefix);

        let papers = paper::Entity::find()
            .filter(paper::Column::DeletedAt.is_null())
            .filter(paper::Column::Title.like(&pattern))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get suggestions: {}", e)))?;

        let suggestions = papers.into_iter().map(|p| p.title).collect();
        Ok(suggestions)
    }

    /// Initialize FTS index for existing papers
    ///
    /// This should be called during migration to populate the FTS index
    pub async fn initialize_fts_index(db: &DatabaseConnection) -> Result<()> {
        info!("Initializing FTS index for existing papers");

        Self::rebuild_fts_index(db).await?;

        Ok(())
    }

    /// Check if FTS index is populated
    ///
    /// Returns the count of papers in the FTS index
    pub async fn check_fts_index_status(db: &DatabaseConnection) -> Result<usize> {
        let pool = db.get_sqlite_connection_pool();

        let row: SqliteRow = sqlx::query("SELECT COUNT(*) as count FROM paper_fts_content")
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::generic(format!("Failed to check FTS index status: {}", e)))?;

        let count: i64 = row.try_get::<i64, _>(0)
            .map_err(|e| AppError::generic(format!("Failed to get count: {}", e)))?;

        info!("FTS index contains {} papers", count);

        Ok(count as usize)
    }

    /// Get sample FTS index entries for debugging
    ///
    /// Returns a few entries from the FTS index to verify content
    pub async fn get_fts_sample(db: &DatabaseConnection) -> Result<Vec<(String, String, String)>> {
        let pool = db.get_sqlite_connection_pool();

        let rows = sqlx::query(
            "SELECT paper_id, title, abstract FROM paper_fts_content LIMIT 5"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::generic(format!("Failed to get FTS sample: {}", e)))?;

        let mut samples = Vec::new();
        for row in rows {
            let paper_id: String = row.try_get::<i64, _>(0).unwrap_or(0).to_string();
            let title: String = row.try_get::<String, _>(1).unwrap_or("".to_string());
            let abstract_text: String = row.try_get::<String, _>(2).unwrap_or("".to_string());
            samples.push((paper_id, title, abstract_text));
        }

        info!("Retrieved {} sample FTS entries", samples.len());
        Ok(samples)
    }

    /// Rebuild the entire FTS index
    ///
    /// This is useful for maintenance or after data corruption
    pub async fn rebuild_fts_index(db: &DatabaseConnection) -> Result<()> {
        info!("Rebuilding FTS index");

        // First, check how many papers we have using sqlx
        let pool = db.get_sqlite_connection_pool();
        let count_result: std::result::Result<SqliteRow, sea_orm::sqlx::Error> =
            sqlx::query("SELECT COUNT(*) as count FROM paper WHERE deleted_at IS NULL")
                .fetch_one(pool)
                .await;

        if let Ok(row) = count_result {
            let count: i64 = row.try_get::<i64, _>(0).unwrap_or(0);
            info!("Found {} non-deleted papers in database", count);
        }

        // Clear existing FTS index content first
        db.execute_unprepared("DELETE FROM paper_fts_content")
            .await
            .map_err(|e| AppError::generic(format!("Failed to clear FTS index: {}", e)))?;
        info!("Cleared existing FTS index content");

        // Re-populate with current data
        db.execute_unprepared(
            r#"
            INSERT INTO paper_fts_content (rowid, paper_id, title, abstract, labels, attachments)
            SELECT
                p.id,
                p.id,
                p.title,
                p.abstract_text,
                (SELECT GROUP_CONCAT(l.name, ' ')
                 FROM label l
                 INNER JOIN paper_label pl ON l.id = pl.label_id
                 WHERE pl.paper_id = p.id),
                (SELECT GROUP_CONCAT(a.file_name, ' ')
                 FROM attachment a
                 WHERE a.paper_id = p.id)
            FROM paper p
            WHERE p.deleted_at IS NULL
            "#,
        )
        .await
        .map_err(|e| AppError::generic(format!("Failed to rebuild FTS index: {}", e)))?;

        info!("FTS content table populated, rebuilding FTS5 virtual table index");

        // Rebuild the FTS5 virtual table index (required for external content tables)
        db.execute_unprepared("INSERT INTO paper_fts(paper_fts) VALUES('rebuild')")
            .await
            .map_err(|e| AppError::generic(format!("Failed to rebuild FTS5 virtual index: {}", e)))?;

        info!("FTS index rebuild completed");
        Ok(())
    }

    /// Normalize BM25 score to 0-100 range
    ///
    /// BM25 returns unbounded scores (lower is better)
    /// We use a sigmoid function to map to 0-100 (higher is better)
    fn normalize_score(raw_score: f64) -> f64 {
        // Use: 100 * sigmoid(-x/scale)
        // For negative raw scores (good matches), we want high normalized scores
        // For positive raw scores (poor matches), we want low normalized scores
        const SCALE: f64 = 5.0;
        let normalized = 100.0 * (-raw_score / SCALE).exp() / (1.0 + (-raw_score / SCALE).exp());
        normalized.clamp(0.0, 100.0)
    }

    /// Check if a string contains Chinese characters
    pub fn contains_chinese(s: &str) -> bool {
        s.chars().any(|c| {
            // Chinese characters are in the range:
            // CJK Unified Ideographs: U+4E00 to U+9FFF
            // CJK Extension A: U+3400 to U+4DBF
            // CJK Extension B: U+20000 to U+2A6DF
            let code = c as u32;
            (0x4E00..=0x9FFF).contains(&code)
                || (0x3400..=0x4DBF).contains(&code)
                || (0x20000..=0x2A6DF).contains(&code)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_score() {
        // Good match (negative BM25 score) should give high normalized score
        let good_score = -20.0;
        let normalized = SearchRepository::normalize_score(good_score);
        assert!(normalized > 90.0);

        // Poor match (positive BM25 score) should give low normalized score
        let poor_score = 20.0;
        let normalized = SearchRepository::normalize_score(poor_score);
        assert!(normalized < 10.0);

        // Neutral score should give middle normalized score
        let neutral_score = 0.0;
        let normalized = SearchRepository::normalize_score(neutral_score);
        assert!((45.0..=55.0).contains(&normalized));
    }
}
