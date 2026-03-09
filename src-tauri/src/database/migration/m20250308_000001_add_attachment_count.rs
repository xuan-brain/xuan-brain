//! Add attachment_count field to paper table for performance optimization
//!
//! This migration adds a denormalized attachment_count field to avoid
//! expensive JOIN queries when loading paper lists.

use sea_orm_migration::prelude::*;

use crate::database::migration::m20240101_000001_initial::Paper;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add attachment_count column with default 0
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .add_column(
                        ColumnDef::new(Paper::AttachmentCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // Initialize attachment_count from existing attachment data
        // Using raw SQL for efficiency
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE paper
                SET attachment_count = (
                    SELECT COUNT(*)
                    FROM attachment
                    WHERE attachment.paper_id = paper.id
                )
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .drop_column(Paper::AttachmentCount)
                    .to_owned(),
            )
            .await
    }
}
