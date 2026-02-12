pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20260122_050159_create_categories;
mod m20260123_161037_create_paper;
mod m20260128_153129_create_paper_category;
mod m20260129_101358_create_paper_detele;
mod m20260131_091618_create_paper_attachment_path;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260122_050159_create_categories::Migration),
            Box::new(m20260123_161037_create_paper::Migration),
            Box::new(m20260128_153129_create_paper_category::Migration),
            Box::new(m20260129_101358_create_paper_detele::Migration),
            Box::new(m20260131_091618_create_paper_attachment_path::Migration),
        ]
    }
}
