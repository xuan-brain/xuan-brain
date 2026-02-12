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
                ALTER TABLE papers ADD COLUMN deleted_at TIMESTAMP DEFAULT NULL;
                CREATE INDEX idx_papers_deleted_at ON papers(deleted_at);
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
