//! Add publisher, issn, language fields to paper table
//!
//! This migration adds fields needed for Zotero import support:
//! - publisher: The publisher of the publication
//! - issn: International Standard Serial Number
//! - language: The language of the paper (e.g., "en", "zh")

use sea_orm_migration::prelude::*;

use crate::database::migration::m20240101_000001_initial::Paper;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .add_column(ColumnDef::new(Paper::Publisher).text())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .add_column(ColumnDef::new(Paper::Issn).text())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .add_column(ColumnDef::new(Paper::Language).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Paper::Table)
                    .drop_column(Paper::Publisher)
                    .drop_column(Paper::Issn)
                    .drop_column(Paper::Language)
                    .to_owned(),
            )
            .await
    }
}
