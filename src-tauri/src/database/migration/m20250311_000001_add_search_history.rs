//! Add search_history table for storing search query history
//!
//! This migration creates a table to store user search history,
//! allowing users to view their past searches.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SearchHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SearchHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SearchHistory::Query).text().not_null())
                    .col(
                        ColumnDef::new(SearchHistory::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_date()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SearchHistory::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SearchHistory {
    Table,
    Id,
    Query,
    CreatedAt,
}
