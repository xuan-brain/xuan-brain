use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        let db = manager.get_connection();
        manager
            .create_table(
                Table::create()
                    .table("category")
                    .if_not_exists()
                    .col(pk_auto("id"))
                    .col(string("name"))
                    .col(integer("parent_id").null())
                    .col(string("ltree_path"))
                    .col(integer("sort_order"))
                    .col(date_time("created_at"))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_category_parent_id")
                            .from("category", "parent_id")
                            .to("category", "id")
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_category_parent_id")
                    .table("category")
                    .col("parent_id")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_category_ltree_path")
                    .table("category")
                    .col("ltree_path")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        Ok(())
    }
}
