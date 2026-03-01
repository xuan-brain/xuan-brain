//! Attachment entity definition

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "attachment")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub paper_id: i64,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Paper,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Paper => Entity::belongs_to(super::paper::Entity)
                .from(Column::PaperId)
                .to(super::paper::Column::Id)
                .into(),
        }
    }
}

impl Related<super::paper::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Paper.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
