//! Add SQLite FTS5 full-text search support for papers
//!
//! This migration creates a virtual table using FTS5 for efficient full-text search
//! across paper title, abstract, labels, and attachments.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Create external content table for FTS5
        // This table stores the pre-processed search data
        manager
            .create_table(
                Table::create()
                    .table(PaperFtsContent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperFtsContent::Rowid)
                            .integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PaperFtsContent::PaperId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PaperFtsContent::Title).text())
                    .col(ColumnDef::new(PaperFtsContent::Abstract).text())
                    .col(ColumnDef::new(PaperFtsContent::Labels).text())
                    .col(ColumnDef::new(PaperFtsContent::Attachments).text())
                    .to_owned(),
            )
            .await?;

        // Create FTS5 virtual table with external content
        // Using unicode61 tokenizer for Chinese and English text support
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                CREATE VIRTUAL TABLE IF NOT EXISTS paper_fts USING fts5(
                    paper_id,
                    title,
                    abstract,
                    labels,
                    attachments,
                    content='paper_fts_content',
                    content_rowid='rowid',
                    tokenize='unicode61'
                )
                "#,
            )
            .await?;

        // Initialize FTS index with existing paper data
        conn.execute_unprepared(
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
        .await?;

        // Create trigger to sync FTS on paper insert
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_insert
            AFTER INSERT ON paper
            BEGIN
                INSERT INTO paper_fts_content (rowid, paper_id, title, abstract, labels, attachments)
                VALUES (
                    NEW.id,
                    NEW.id,
                    NEW.title,
                    NEW.abstract_text,
                    (SELECT GROUP_CONCAT(l.name, ' ')
                     FROM label l
                     INNER JOIN paper_label pl ON l.id = pl.label_id
                     WHERE pl.paper_id = NEW.id),
                    (SELECT GROUP_CONCAT(a.file_name, ' ')
                     FROM attachment a
                     WHERE a.paper_id = NEW.id)
                );
            END
            "#,
        )
        .await?;

        // Create trigger to sync FTS on paper update
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_update
            AFTER UPDATE OF title, abstract_text ON paper
            BEGIN
                UPDATE paper_fts_content
                SET title = NEW.title,
                    abstract = NEW.abstract_text
                WHERE paper_id = NEW.id;
            END
            "#,
        )
        .await?;

        // Create trigger to sync FTS on paper delete
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_delete
            AFTER DELETE ON paper
            BEGIN
                DELETE FROM paper_fts_content WHERE paper_id = OLD.id;
            END
            "#,
        )
        .await?;

        // Create trigger to sync labels when paper_label is inserted
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_label_insert
            AFTER INSERT ON paper_label
            BEGIN
                UPDATE paper_fts_content
                SET labels = (SELECT GROUP_CONCAT(l.name, ' ')
                              FROM label l
                              INNER JOIN paper_label pl ON l.id = pl.label_id
                              WHERE pl.paper_id = NEW.paper_id)
                WHERE paper_id = NEW.paper_id;
            END
            "#,
        )
        .await?;

        // Create trigger to sync labels when paper_label is deleted
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_label_delete
            AFTER DELETE ON paper_label
            BEGIN
                UPDATE paper_fts_content
                SET labels = (SELECT GROUP_CONCAT(l.name, ' ')
                              FROM label l
                              INNER JOIN paper_label pl ON l.id = pl.label_id
                              WHERE pl.paper_id = OLD.paper_id)
                WHERE paper_id = OLD.paper_id;
            END
            "#,
        )
        .await?;

        // Create trigger to sync attachments when attachment is inserted
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_attachment_insert
            AFTER INSERT ON attachment
            BEGIN
                UPDATE paper_fts_content
                SET attachments = (SELECT GROUP_CONCAT(a.file_name, ' ')
                                   FROM attachment a
                                   WHERE a.paper_id = NEW.paper_id)
                WHERE paper_id = NEW.paper_id;
            END
            "#,
        )
        .await?;

        // Create trigger to sync attachments when attachment is deleted
        conn.execute_unprepared(
            r#"
            CREATE TRIGGER IF NOT EXISTS paper_fts_attachment_delete
            AFTER DELETE ON attachment
            BEGIN
                UPDATE paper_fts_content
                SET attachments = (SELECT GROUP_CONCAT(a.file_name, ' ')
                                   FROM attachment a
                                   WHERE a.paper_id = OLD.paper_id)
                WHERE paper_id = OLD.paper_id;
            END
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        // Drop all triggers first
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_insert")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_update")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_delete")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_label_insert")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_label_delete")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_attachment_insert")
            .await?;
        conn.execute_unprepared("DROP TRIGGER IF EXISTS paper_fts_attachment_delete")
            .await?;

        // Drop FTS virtual table
        conn.execute_unprepared("DROP TABLE IF EXISTS paper_fts")
            .await?;

        // Drop content table
        manager
            .drop_table(Table::drop().table(PaperFtsContent::Table).to_owned())
            .await?;

        Ok(())
    }
}

// FTS content table enum
#[derive(Iden)]
enum PaperFtsContent {
    Table,
    Rowid,
    PaperId,
    Title,
    Abstract,
    Labels,
    Attachments,
}
