//! Clip-Label relationship entity

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "clip_label")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub clipping_id: i64,
    pub label_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Clipping,
    Label,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Clipping => Entity::belongs_to(super::clipping::Entity)
                .from(Column::ClippingId)
                .to(super::clipping::Column::Id)
                .into(),
            Self::Label => Entity::belongs_to(super::label::Entity)
                .from(Column::LabelId)
                .to(super::label::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
