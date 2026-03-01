//! Paper-Category relationship entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "paper_category")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub paper_id: i64,
    pub category_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Paper,
    Category,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Paper => Entity::belongs_to(super::paper::Entity)
                .from(Column::PaperId)
                .to(super::paper::Column::Id)
                .into(),
            Self::Category => Entity::belongs_to(super::category::Entity)
                .from(Column::CategoryId)
                .to(super::category::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
