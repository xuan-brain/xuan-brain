//! Database migration module

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::*;

mod m20240101_000001_initial;
mod m20240307_000001_split_author_name;
mod m20250307_000002_add_paper_fields;

#[allow(unused_imports)]
pub use m20240101_000001_initial::Migration as InitialMigration;
pub use m20240307_000001_split_author_name::Migration as SplitAuthorNameMigration;
pub use m20250307_000002_add_paper_fields::Migration as AddPaperFieldsMigration;

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
        ]
    }
}
