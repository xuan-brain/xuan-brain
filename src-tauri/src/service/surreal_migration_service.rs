//! SurrealDB Migration Service
//!
//! Provides functionality to migrate data from SQLite (SeaORM) to SurrealDB 3.0.
//! This service handles the migration of all entities while preserving relationships.

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::database::entities::{
    attachments, authors, category, keywords, label, paper_authors, paper_category, paper_labels,
    papers,
};
use crate::surreal::connection::SurrealClient;
use crate::sys::error::{AppError, Result};

/// Migration report containing statistics about the migration process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MigrationReport {
    pub papers_migrated: usize,
    pub authors_migrated: usize,
    pub keywords_migrated: usize,
    pub labels_migrated: usize,
    pub categories_migrated: usize,
    pub attachments_migrated: usize,
    pub paper_author_relations: usize,
    pub paper_label_relations: usize,
    pub paper_category_relations: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}


/// Database migrator from SQLite to SurrealDB
pub struct SurrealMigrator {
    sqlite: Arc<DatabaseConnection>,
    surreal: Arc<SurrealClient>,
}

impl SurrealMigrator {
    pub fn new(sqlite: Arc<DatabaseConnection>, surreal: Arc<SurrealClient>) -> Self {
        Self { sqlite, surreal }
    }

    /// Run the full migration process
    #[instrument(skip(self))]
    pub async fn migrate_all(&self) -> Result<MigrationReport> {
        let start = std::time::Instant::now();
        let mut report = MigrationReport::default();

        info!("Starting SQLite to SurrealDB migration...");

        // Phase 1: Migrate independent entities first
        match self.migrate_labels().await {
            Ok(count) => {
                report.labels_migrated = count;
                info!("Migrated {} labels", count);
            }
            Err(e) => {
                report.errors.push(format!("Label migration failed: {}", e));
                warn!("Label migration failed: {}", e);
            }
        }

        match self.migrate_keywords().await {
            Ok(count) => {
                report.keywords_migrated = count;
                info!("Migrated {} keywords", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Keyword migration failed: {}", e));
                warn!("Keyword migration failed: {}", e);
            }
        }

        match self.migrate_authors().await {
            Ok(count) => {
                report.authors_migrated = count;
                info!("Migrated {} authors", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Author migration failed: {}", e));
                warn!("Author migration failed: {}", e);
            }
        }

        match self.migrate_categories().await {
            Ok(count) => {
                report.categories_migrated = count;
                info!("Migrated {} categories", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Category migration failed: {}", e));
                warn!("Category migration failed: {}", e);
            }
        }

        // Phase 2: Migrate main entity (papers)
        match self.migrate_papers().await {
            Ok(count) => {
                report.papers_migrated = count;
                info!("Migrated {} papers", count);
            }
            Err(e) => {
                report.errors.push(format!("Paper migration failed: {}", e));
                warn!("Paper migration failed: {}", e);
            }
        }

        match self.migrate_attachments().await {
            Ok(count) => {
                report.attachments_migrated = count;
                info!("Migrated {} attachments", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Attachment migration failed: {}", e));
                warn!("Attachment migration failed: {}", e);
            }
        }

        // Phase 3: Migrate relationships
        match self.migrate_paper_authors().await {
            Ok(count) => {
                report.paper_author_relations = count;
                info!("Migrated {} paper-author relations", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Paper-author migration failed: {}", e));
                warn!("Paper-author migration failed: {}", e);
            }
        }

        match self.migrate_paper_labels().await {
            Ok(count) => {
                report.paper_label_relations = count;
                info!("Migrated {} paper-label relations", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Paper-label migration failed: {}", e));
                warn!("Paper-label migration failed: {}", e);
            }
        }

        match self.migrate_paper_categories().await {
            Ok(count) => {
                report.paper_category_relations = count;
                info!("Migrated {} paper-category relations", count);
            }
            Err(e) => {
                report
                    .errors
                    .push(format!("Paper-category migration failed: {}", e));
                warn!("Paper-category migration failed: {}", e);
            }
        }

        report.duration_ms = start.elapsed().as_millis() as u64;
        info!("Migration completed in {}ms", report.duration_ms);

        Ok(report)
    }

    /// Migrate labels from SQLite to SurrealDB
    async fn migrate_labels(&self) -> Result<usize> {
        let labels = label::Entity::find()
            .order_by_asc(label::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for lbl in labels {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        name = $name,
                        color = $color,
                        document_count = $document_count,
                        created_at = time::now()
                    "#,
                )
                .bind(("id", format!("label:{}", lbl.id)))
                .bind(("name", lbl.name))
                .bind(("color", lbl.color))
                .bind(("document_count", lbl.document_count.unwrap_or(0) as i32))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_labels", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate keywords from SQLite to SurrealDB
    async fn migrate_keywords(&self) -> Result<usize> {
        let keywords = keywords::Entity::find()
            .order_by_asc(keywords::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for kw in keywords {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        word = $word
                    "#,
                )
                .bind(("id", format!("keyword:{}", kw.id)))
                .bind(("word", kw.word))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_keywords", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate authors from SQLite to SurrealDB
    async fn migrate_authors(&self) -> Result<usize> {
        let authors = authors::Entity::find()
            .order_by_asc(authors::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for author in authors {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        name = $name,
                        affiliation = $affiliation,
                        email = $email,
                        created_at = time::now()
                    "#,
                )
                .bind(("id", format!("author:{}", author.id)))
                .bind(("name", author.name))
                .bind(("affiliation", author.affiliation))
                .bind(("email", author.email))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_authors", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate categories from SQLite to SurrealDB (preserving hierarchy)
    async fn migrate_categories(&self) -> Result<usize> {
        let categories = category::Entity::find()
            .order_by_asc(category::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        // First pass: create all categories without parent
        for cat in &categories {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        name = $name,
                        parent = NONE,
                        sort_order = $sort_order,
                        created_at = time::now()
                    "#,
                )
                .bind(("id", format!("category:{}", cat.id)))
                .bind(("name", cat.name.clone()))
                .bind(("sort_order", cat.sort_order as i32))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_categories", e.to_string()))?;
            count += 1;
        }

        // Second pass: update parent references
        for cat in &categories {
            if let Some(parent_id) = cat.parent_id {
                self.surreal
                    .query("UPDATE type::record($id) SET parent = type::record($parent)")
                    .bind(("id", format!("category:{}", cat.id)))
                    .bind(("parent", format!("category:{}", parent_id)))
                    .await
                    .map_err(|e| {
                        AppError::surrealdb_error("migrate_categories_parent", e.to_string())
                    })?;
            }
        }

        Ok(count)
    }

    /// Migrate papers from SQLite to SurrealDB
    async fn migrate_papers(&self) -> Result<usize> {
        let papers = papers::Entity::find()
            .filter(papers::Column::DeletedAt.is_null())
            .order_by_asc(papers::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for paper in papers {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        title = $title,
                        `abstract` = $abstract,
                        doi = $doi,
                        publication_year = $publication_year,
                        publication_date = $publication_date,
                        journal_name = $journal_name,
                        conference_name = $conference_name,
                        volume = $volume,
                        issue = $issue,
                        pages = $pages,
                        url = $url,
                        citation_count = $citation_count,
                        read_status = $read_status,
                        notes = $notes,
                        attachment_path = $attachment_path,
                        created_at = time::now(),
                        updated_at = time::now(),
                        deleted_at = NONE
                    "#,
                )
                .bind(("id", format!("paper:{}", paper.id)))
                .bind(("title", paper.title))
                .bind(("abstract", paper.r#abstract))
                .bind(("doi", paper.doi))
                .bind(("publication_year", paper.publication_year.map(|y| y as i32)))
                .bind(("publication_date", paper.publication_date))
                .bind(("journal_name", paper.journal_name))
                .bind(("conference_name", paper.conference_name))
                .bind(("volume", paper.volume))
                .bind(("issue", paper.issue))
                .bind(("pages", paper.pages))
                .bind(("url", paper.url))
                .bind(("citation_count", paper.citation_count.unwrap_or(0) as i32))
                .bind((
                    "read_status",
                    paper.read_status.unwrap_or_else(|| "unread".to_string()),
                ))
                .bind(("notes", paper.notes))
                .bind(("attachment_path", paper.attachment_path))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_papers", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate attachments from SQLite to SurrealDB
    async fn migrate_attachments(&self) -> Result<usize> {
        let attachments = attachments::Entity::find()
            .order_by_asc(attachments::Column::Id)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for att in attachments {
            self.surreal
                .query(
                    r#"
                    CREATE type::record($id) SET
                        paper = type::record($paper),
                        file_type = $file_type,
                        file_name = $file_name,
                        file_path = NONE,
                        file_size = NONE,
                        created_at = time::now()
                    "#,
                )
                .bind(("id", format!("attachment:{}", att.id)))
                .bind(("paper", format!("paper:{}", att.paper_id)))
                .bind(("file_type", att.file_type))
                .bind(("file_name", att.file_name))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_attachments", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate paper-author relationships
    async fn migrate_paper_authors(&self) -> Result<usize> {
        let relations = paper_authors::Entity::find()
            .order_by_asc(paper_authors::Column::PaperId)
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for rel in relations {
            self.surreal
                .query(
                    r#"
                    LET $paper_id = type::record($paper);
                    LET $author_id = type::record($author);
                    RELATE $paper_id->paper_author->$author_id SET
                        author_order = $author_order,
                        is_corresponding = $is_corresponding
                    "#,
                )
                .bind(("paper", format!("paper:{}", rel.paper_id)))
                .bind(("author", format!("author:{}", rel.author_id)))
                .bind(("author_order", rel.author_order as i32))
                .bind(("is_corresponding", rel.is_corresponding.unwrap_or(false)))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_paper_authors", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate paper-label relationships
    async fn migrate_paper_labels(&self) -> Result<usize> {
        let relations = paper_labels::Entity::find()
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for rel in relations {
            self.surreal
                .query(
                    r#"
                    LET $paper_id = type::record($paper);
                    LET $label_id = type::record($label);
                    RELATE $paper_id->paper_label->$label_id
                    "#,
                )
                .bind(("paper", format!("paper:{}", rel.paper_id)))
                .bind(("label", format!("label:{}", rel.label_id)))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_paper_labels", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Migrate paper-category relationships
    async fn migrate_paper_categories(&self) -> Result<usize> {
        let relations = paper_category::Entity::find()
            .all(self.sqlite.as_ref())
            .await?;

        let mut count = 0;
        for rel in relations {
            self.surreal
                .query(
                    r#"
                    LET $paper_id = type::record($paper);
                    LET $category_id = type::record($category);
                    RELATE $paper_id->paper_category->$category_id
                    "#,
                )
                .bind(("paper", format!("paper:{}", rel.paper_id)))
                .bind(("category", format!("category:{}", rel.category_id)))
                .await
                .map_err(|e| AppError::surrealdb_error("migrate_paper_categories", e.to_string()))?;
            count += 1;
        }

        Ok(count)
    }

    /// Verify migration by comparing counts
    pub async fn verify_counts(&self) -> Result<MigrationReport> {
        let mut report = MigrationReport::default();

        // Count papers in SurrealDB
        let paper_count: Option<i64> = self
            .surreal
            .query("SELECT count() FROM paper GROUP ALL")
            .await
            .map_err(|e| AppError::surrealdb_error("verify_papers", e.to_string()))?
            .take((0, "count"))
            .map_err(|e| AppError::surrealdb_error("verify_papers_result", e.to_string()))?;

        report.papers_migrated = paper_count.unwrap_or(0) as usize;

        // Count labels
        let label_count: Option<i64> = self
            .surreal
            .query("SELECT count() FROM label GROUP ALL")
            .await
            .map_err(|e| AppError::surrealdb_error("verify_labels", e.to_string()))?
            .take((0, "count"))
            .map_err(|e| AppError::surrealdb_error("verify_labels_result", e.to_string()))?;

        report.labels_migrated = label_count.unwrap_or(0) as usize;

        // Count categories
        let category_count: Option<i64> = self
            .surreal
            .query("SELECT count() FROM category GROUP ALL")
            .await
            .map_err(|e| AppError::surrealdb_error("verify_categories", e.to_string()))?
            .take((0, "count"))
            .map_err(|e| AppError::surrealdb_error("verify_categories_result", e.to_string()))?;

        report.categories_migrated = category_count.unwrap_or(0) as usize;

        Ok(report)
    }
}
