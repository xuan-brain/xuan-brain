pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20260122_050159_create_categories;
mod m20260123_161037_create_paper;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260122_050159_create_categories::Migration),
            Box::new(m20260123_161037_create_paper::Migration),
        ]
    }
}
