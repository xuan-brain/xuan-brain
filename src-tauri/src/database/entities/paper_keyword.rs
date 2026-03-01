//! Paper-Keyword relationship entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "paper_keyword")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub paper_id: i64,
    pub keyword_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Paper,
    Keyword,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Paper => Entity::belongs_to(super::paper::Entity)
                .from(Column::PaperId)
                .to(super::paper::Column::Id)
                .into(),
            Self::Keyword => Entity::belongs_to(super::keyword::Entity)
                .from(Column::KeywordId)
                .to(super::keyword::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
