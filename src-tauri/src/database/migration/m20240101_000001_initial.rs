//! Initial database migration
//!
//! Creates all tables for the xuan-brain application.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create paper table
        manager
            .create_table(
                Table::create()
                    .table(Paper::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Paper::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Paper::Title).text().not_null())
                    .col(ColumnDef::new(Paper::AbstractText).text())
                    .col(ColumnDef::new(Paper::Doi).text().unique_key())
                    .col(ColumnDef::new(Paper::PublicationYear).integer())
                    .col(ColumnDef::new(Paper::PublicationDate).text())
                    .col(ColumnDef::new(Paper::JournalName).text())
                    .col(ColumnDef::new(Paper::ConferenceName).text())
                    .col(ColumnDef::new(Paper::Volume).text())
                    .col(ColumnDef::new(Paper::Issue).text())
                    .col(ColumnDef::new(Paper::Pages).text())
                    .col(ColumnDef::new(Paper::Url).text())
                    .col(ColumnDef::new(Paper::CitationCount).integer().default(0))
                    .col(ColumnDef::new(Paper::ReadStatus).text().default("unread"))
                    .col(ColumnDef::new(Paper::Notes).text())
                    .col(ColumnDef::new(Paper::AttachmentPath).text())
                    .col(ColumnDef::new(Paper::CreatedAt).text().not_null())
                    .col(ColumnDef::new(Paper::UpdatedAt).text().not_null())
                    .col(ColumnDef::new(Paper::DeletedAt).text())
                    .to_owned(),
            )
            .await?;

        // Create author table
        manager
            .create_table(
                Table::create()
                    .table(Author::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Author::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Author::Name).text().not_null())
                    .col(ColumnDef::new(Author::Affiliation).text())
                    .col(ColumnDef::new(Author::Email).text())
                    .col(ColumnDef::new(Author::CreatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create keyword table
        manager
            .create_table(
                Table::create()
                    .table(Keyword::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Keyword::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Keyword::Word).text().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        // Create label table
        manager
            .create_table(
                Table::create()
                    .table(Label::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Label::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Label::Name).text().not_null())
                    .col(ColumnDef::new(Label::Color).text().default("#1976D2"))
                    .col(ColumnDef::new(Label::DocumentCount).integer().default(0))
                    .col(ColumnDef::new(Label::CreatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create category table
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Category::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Category::Name).text().not_null())
                    .col(ColumnDef::new(Category::ParentId).integer())
                    .col(ColumnDef::new(Category::SortOrder).integer().default(0))
                    .col(ColumnDef::new(Category::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_category_parent")
                            .from(Category::Table, Category::ParentId)
                            .to(Category::Table, Category::Id)
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Create attachment table
        manager
            .create_table(
                Table::create()
                    .table(Attachment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Attachment::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Attachment::PaperId).integer().not_null())
                    .col(ColumnDef::new(Attachment::FileName).text())
                    .col(ColumnDef::new(Attachment::FileType).text())
                    .col(ColumnDef::new(Attachment::FileSize).integer())
                    .col(ColumnDef::new(Attachment::CreatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_attachment_paper")
                            .from(Attachment::Table, Attachment::PaperId)
                            .to(Paper::Table, Paper::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create clipping table
        manager
            .create_table(
                Table::create()
                    .table(Clipping::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Clipping::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Clipping::Title).text().not_null())
                    .col(ColumnDef::new(Clipping::Url).text().not_null().unique_key())
                    .col(ColumnDef::new(Clipping::Content).text())
                    .col(ColumnDef::new(Clipping::SourceDomain).text())
                    .col(ColumnDef::new(Clipping::Author).text())
                    .col(ColumnDef::new(Clipping::PublishedDate).text())
                    .col(ColumnDef::new(Clipping::Excerpt).text())
                    .col(ColumnDef::new(Clipping::ThumbnailUrl).text())
                    .col(ColumnDef::new(Clipping::ReadStatus).integer().default(0))
                    .col(ColumnDef::new(Clipping::Notes).text())
                    .col(ColumnDef::new(Clipping::Tags).text())
                    .col(ColumnDef::new(Clipping::ImagePaths).text())
                    .col(ColumnDef::new(Clipping::CreatedAt).text().not_null())
                    .col(ColumnDef::new(Clipping::UpdatedAt).text().not_null())
                    .to_owned(),
            )
            .await?;

        // Create comment table
        manager
            .create_table(
                Table::create()
                    .table(Comment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Comment::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Comment::ClippingId).integer().not_null())
                    .col(ColumnDef::new(Comment::Content).text().not_null())
                    .col(ColumnDef::new(Comment::CreatedAt).text().not_null())
                    .col(ColumnDef::new(Comment::UpdatedAt).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_clipping")
                            .from(Comment::Table, Comment::ClippingId)
                            .to(Clipping::Table, Clipping::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create paper_author relationship table
        manager
            .create_table(
                Table::create()
                    .table(PaperAuthor::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperAuthor::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PaperAuthor::PaperId).integer().not_null())
                    .col(ColumnDef::new(PaperAuthor::AuthorId).integer().not_null())
                    .col(ColumnDef::new(PaperAuthor::AuthorOrder).integer().default(0))
                    .col(ColumnDef::new(PaperAuthor::IsCorresponding).integer().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_author_paper")
                            .from(PaperAuthor::Table, PaperAuthor::PaperId)
                            .to(Paper::Table, Paper::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_author_author")
                            .from(PaperAuthor::Table, PaperAuthor::AuthorId)
                            .to(Author::Table, Author::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_paper_author_unique")
                            .table(PaperAuthor::Table)
                            .col(PaperAuthor::PaperId)
                            .col(PaperAuthor::AuthorId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create paper_keyword relationship table
        manager
            .create_table(
                Table::create()
                    .table(PaperKeyword::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperKeyword::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PaperKeyword::PaperId).integer().not_null())
                    .col(ColumnDef::new(PaperKeyword::KeywordId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_keyword_paper")
                            .from(PaperKeyword::Table, PaperKeyword::PaperId)
                            .to(Paper::Table, Paper::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_keyword_keyword")
                            .from(PaperKeyword::Table, PaperKeyword::KeywordId)
                            .to(Keyword::Table, Keyword::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_paper_keyword_unique")
                            .table(PaperKeyword::Table)
                            .col(PaperKeyword::PaperId)
                            .col(PaperKeyword::KeywordId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create paper_label relationship table
        manager
            .create_table(
                Table::create()
                    .table(PaperLabel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperLabel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PaperLabel::PaperId).integer().not_null())
                    .col(ColumnDef::new(PaperLabel::LabelId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_label_paper")
                            .from(PaperLabel::Table, PaperLabel::PaperId)
                            .to(Paper::Table, Paper::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_label_label")
                            .from(PaperLabel::Table, PaperLabel::LabelId)
                            .to(Label::Table, Label::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_paper_label_unique")
                            .table(PaperLabel::Table)
                            .col(PaperLabel::PaperId)
                            .col(PaperLabel::LabelId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create paper_category relationship table
        manager
            .create_table(
                Table::create()
                    .table(PaperCategory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperCategory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PaperCategory::PaperId).integer().not_null())
                    .col(ColumnDef::new(PaperCategory::CategoryId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_category_paper")
                            .from(PaperCategory::Table, PaperCategory::PaperId)
                            .to(Paper::Table, Paper::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_category_category")
                            .from(PaperCategory::Table, PaperCategory::CategoryId)
                            .to(Category::Table, Category::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_paper_category_unique")
                            .table(PaperCategory::Table)
                            .col(PaperCategory::PaperId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create clip_label relationship table
        manager
            .create_table(
                Table::create()
                    .table(ClipLabel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClipLabel::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ClipLabel::ClippingId).integer().not_null())
                    .col(ColumnDef::new(ClipLabel::LabelId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_clip_label_clipping")
                            .from(ClipLabel::Table, ClipLabel::ClippingId)
                            .to(Clipping::Table, Clipping::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_clip_label_label")
                            .from(ClipLabel::Table, ClipLabel::LabelId)
                            .to(Label::Table, Label::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_clip_label_unique")
                            .table(ClipLabel::Table)
                            .col(ClipLabel::ClippingId)
                            .col(ClipLabel::LabelId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_paper_doi")
                    .table(Paper::Table)
                    .col(Paper::Doi)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_paper_deleted_at")
                    .table(Paper::Table)
                    .col(Paper::DeletedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_paper_created_at")
                    .table(Paper::Table)
                    .col(Paper::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_category_parent")
                    .table(Category::Table)
                    .col(Category::ParentId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_clipping_url")
                    .table(Clipping::Table)
                    .col(Clipping::Url)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_clipping_source_domain")
                    .table(Clipping::Table)
                    .col(Clipping::SourceDomain)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_comment_clipping_id")
                    .table(Comment::Table)
                    .col(Comment::ClippingId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_attachment_paper_id")
                    .table(Attachment::Table)
                    .col(Attachment::PaperId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(ClipLabel::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaperCategory::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaperLabel::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaperKeyword::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PaperAuthor::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Comment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Clipping::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Attachment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Label::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Keyword::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Author::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Paper::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Paper table
#[derive(Iden)]
enum Paper {
    Table,
    Id,
    Title,
    AbstractText,
    Doi,
    PublicationYear,
    PublicationDate,
    JournalName,
    ConferenceName,
    Volume,
    Issue,
    Pages,
    Url,
    CitationCount,
    ReadStatus,
    Notes,
    AttachmentPath,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

// Author table
#[derive(Iden)]
enum Author {
    Table,
    Id,
    Name,
    Affiliation,
    Email,
    CreatedAt,
}

// Keyword table
#[derive(Iden)]
enum Keyword {
    Table,
    Id,
    Word,
}

// Label table
#[derive(Iden)]
enum Label {
    Table,
    Id,
    Name,
    Color,
    DocumentCount,
    CreatedAt,
}

// Category table
#[derive(Iden)]
enum Category {
    Table,
    Id,
    Name,
    ParentId,
    SortOrder,
    CreatedAt,
}

// Attachment table
#[derive(Iden)]
enum Attachment {
    Table,
    Id,
    PaperId,
    FileName,
    FileType,
    FileSize,
    CreatedAt,
}

// Clipping table
#[derive(Iden)]
enum Clipping {
    Table,
    Id,
    Title,
    Url,
    Content,
    SourceDomain,
    Author,
    PublishedDate,
    Excerpt,
    ThumbnailUrl,
    ReadStatus,
    Notes,
    Tags,
    ImagePaths,
    CreatedAt,
    UpdatedAt,
}

// Comment table
#[derive(Iden)]
enum Comment {
    Table,
    Id,
    ClippingId,
    Content,
    CreatedAt,
    UpdatedAt,
}

// Paper-Author relationship table
#[derive(Iden)]
enum PaperAuthor {
    Table,
    Id,
    PaperId,
    AuthorId,
    AuthorOrder,
    IsCorresponding,
}

// Paper-Keyword relationship table
#[derive(Iden)]
enum PaperKeyword {
    Table,
    Id,
    PaperId,
    KeywordId,
}

// Paper-Label relationship table
#[derive(Iden)]
enum PaperLabel {
    Table,
    Id,
    PaperId,
    LabelId,
}

// Paper-Category relationship table
#[derive(Iden)]
enum PaperCategory {
    Table,
    Id,
    PaperId,
    CategoryId,
}

// Clip-Label relationship table
#[derive(Iden)]
enum ClipLabel {
    Table,
    Id,
    ClippingId,
    LabelId,
}
