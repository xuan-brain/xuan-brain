//! Comment entity definition

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub clipping_id: i64,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Clipping,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Clipping => Entity::belongs_to(super::clipping::Entity)
                .from(Column::ClippingId)
                .to(super::clipping::Column::Id)
                .into(),
        }
    }
}

impl Related<super::clipping::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Clipping.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
