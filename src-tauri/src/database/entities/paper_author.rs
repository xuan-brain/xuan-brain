//! Paper-Author relationship entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "paper_author")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub paper_id: i64,
    pub author_id: i64,
    pub author_order: i32,
    pub is_corresponding: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Paper,
    Author,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Paper => Entity::belongs_to(super::paper::Entity)
                .from(Column::PaperId)
                .to(super::paper::Column::Id)
                .into(),
            Self::Author => Entity::belongs_to(super::author::Entity)
                .from(Column::AuthorId)
                .to(super::author::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
