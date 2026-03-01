//! Database migration module

use sea_orm::DatabaseConnection;
use sea_orm_migration::prelude::*;

mod m20240101_000001_initial;

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
        vec![Box::new(m20240101_000001_initial::Migration)]
    }
}
