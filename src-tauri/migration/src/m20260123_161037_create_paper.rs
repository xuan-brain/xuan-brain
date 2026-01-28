use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();
        db.execute_unprepared(
            "
                -- 开启外键约束
                PRAGMA foreign_keys = ON;

                -- ==========================================
                -- 1. 论文主表
                -- ==========================================
                CREATE TABLE papers (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    abstract TEXT,
                    doi TEXT UNIQUE,
                    publication_year INTEGER,
                    publication_date DATE,
                    journal_name TEXT,
                    conference_name TEXT,
                    volume TEXT,
                    issue TEXT,
                    pages TEXT,
                    url TEXT,
                    citation_count INTEGER DEFAULT 0,
                    read_status TEXT DEFAULT 'unread',
                    notes TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                );

                -- ==========================================
                -- 2. 作者模块
                -- ==========================================
                CREATE TABLE authors (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    affiliation TEXT,
                    email TEXT
                );

                CREATE TABLE paper_authors (
                    paper_id INTEGER,
                    author_id INTEGER,
                    author_order INTEGER NOT NULL DEFAULT 0,
                    is_corresponding BOOLEAN DEFAULT 0, -- 是否为通讯作者
                    correspondence_email TEXT,
                    PRIMARY KEY (paper_id, author_id),
                    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
                    FOREIGN KEY (author_id) REFERENCES authors(id) ON DELETE CASCADE
                );

                -- ==========================================
                -- 3. 标签模块 (使用您的指定结构)
                -- ==========================================

                CREATE TABLE paper_labels (
                    paper_id INTEGER,
                    label_id INTEGER,
                    PRIMARY KEY (paper_id, label_id),
                    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
                    FOREIGN KEY (label_id) REFERENCES label(id) ON DELETE CASCADE
                );

                -- 标签计数自动维护触发器
                CREATE TRIGGER increment_label_count
                AFTER INSERT ON paper_labels
                FOR EACH ROW
                BEGIN
                    UPDATE label
                    SET document_count = document_count + 1,
                        updated_at = datetime('now')
                    WHERE id = NEW.label_id;
                END;

                CREATE TRIGGER decrement_label_count
                AFTER DELETE ON paper_labels
                FOR EACH ROW
                BEGIN
                    UPDATE label
                    SET document_count = document_count - 1,
                        updated_at = datetime('now')
                    WHERE id = OLD.label_id;
                END;

                -- ==========================================
                -- 4. 关键词模块
                -- ==========================================
                CREATE TABLE keywords (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    word TEXT UNIQUE NOT NULL
                );

                CREATE TABLE paper_keywords (
                    paper_id INTEGER,
                    keyword_id INTEGER,
                    PRIMARY KEY (paper_id, keyword_id),
                    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
                    FOREIGN KEY (keyword_id) REFERENCES keywords(id) ON DELETE CASCADE
                );

                -- ==========================================
                -- 5. 附件模块
                -- ==========================================
                CREATE TABLE attachments (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    paper_id INTEGER NOT NULL,
                    file_path TEXT NOT NULL,
                    file_type TEXT,
                    file_name TEXT,
                    description TEXT,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE
                );

                -- ==========================================
                -- 6. 索引优化
                -- ==========================================
                CREATE INDEX idx_papers_title ON papers(title);
                CREATE INDEX idx_papers_year ON papers(publication_year);
                CREATE INDEX idx_authors_name ON authors(name);
                CREATE INDEX idx_paper_authors_pid ON paper_authors(paper_id);
                CREATE INDEX idx_label_name ON label(name);

            ",
        )
        .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        Ok(())
    }
}
