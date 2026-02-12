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
                -- 1. 论文 分类 模块
                -- ==========================================

                CREATE TABLE paper_category (
                    paper_id INTEGER,
                    category_id INTEGER,
                    PRIMARY KEY (paper_id, category_id),
                    FOREIGN KEY (paper_id) REFERENCES papers(id) ON DELETE CASCADE,
                    FOREIGN KEY (category_id) REFERENCES category(id) ON DELETE CASCADE
                );

                -- ==========================================
                -- 2. 索引优化
                -- ==========================================
                CREATE INDEX idx_papers_category ON paper_category(category_id);
            ",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        Ok(())
    }
}
