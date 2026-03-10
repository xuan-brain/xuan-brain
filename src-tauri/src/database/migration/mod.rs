//! Database migration module

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::*;

mod m20240101_000001_initial;
mod m20240307_000001_split_author_name;
mod m20250307_000002_add_paper_fields;
mod m20250308_000001_add_attachment_count;
mod m20250309_000001_add_fts5_search;
mod m20250310_000001_update_fts5_tokenizer;
mod m20250311_000001_add_search_history;

#[allow(unused_imports)]
pub use m20240101_000001_initial::Migration as InitialMigration;

/// Run all pending migrations
pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Run migrations
    Migrator::up(db, None).await?;

    Ok(())
}

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_initial::Migration),
            // Box::new(m20240307_000001_split_author_name::Migration),
            Box::new(m20250307_000002_add_paper_fields::Migration),
            Box::new(m20250308_000001_add_attachment_count::Migration),
            Box::new(m20250309_000001_add_fts5_search::Migration),
            Box::new(m20250310_000001_update_fts5_tokenizer::Migration),
            Box::new(m20250311_000001_add_search_history::Migration),
        ]
    }
}
