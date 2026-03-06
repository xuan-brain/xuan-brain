//! Migration: Split author name into first_name and last_name
//!
//! This migration splits the single `name` field in the author table into
//! `first_name` (required) and `last_name` (optional) to better support
//! both Western and Chinese name formats.
//!
//! Migration strategy:
//! - Existing `name` values are copied to `first_name`
//! - `last_name` is set to NULL for all existing records
//! - Users can manually edit to split names if needed

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add first_name column (nullable initially)
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .add_column(ColumnDef::new(Author::FirstName).text())
                    .to_owned(),
            )
            .await?;

        // Step 2: Add last_name column (nullable)
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .add_column(ColumnDef::new(Author::LastName).text())
                    .to_owned(),
            )
            .await?;

        // Step 3: Migrate existing data - copy name to first_name
        // This is the safest approach: existing names go entirely into first_name
        // Users can manually split them later if needed
        manager
            .get_connection()
            .execute_unprepared("UPDATE author SET first_name = name WHERE first_name IS NULL")
            .await?;

        // Step 4: Make first_name NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .modify_column(ColumnDef::new(Author::FirstName).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Step 5: Drop the old name column
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .drop_column(Author::Name)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Step 1: Add name column back (nullable initially)
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .add_column(ColumnDef::new(Author::Name).text())
                    .to_owned(),
            )
            .await?;

        // Step 2: Migrate data back - combine first_name and last_name
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE author SET name = first_name || ' ' || COALESCE(last_name, '') WHERE name IS NULL",
            )
            .await?;

        // Step 3: Clean up trailing spaces (for records without last_name)
        manager
            .get_connection()
            .execute_unprepared("UPDATE author SET name = TRIM(name)")
            .await?;

        // Step 4: Make name NOT NULL
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .modify_column(ColumnDef::new(Author::Name).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Step 5: Drop first_name column
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .drop_column(Author::FirstName)
                    .to_owned(),
            )
            .await?;

        // Step 6: Drop last_name column
        manager
            .alter_table(
                Table::alter()
                    .table(Author::Table)
                    .drop_column(Author::LastName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Author {
    Table,
    #[iden = "name"]
    Name,
    #[iden = "first_name"]
    FirstName,
    #[iden = "last_name"]
    LastName,
}
