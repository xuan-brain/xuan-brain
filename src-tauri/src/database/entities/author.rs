//! Author entity definition

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "author")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// First name / Given name / 名 (for Chinese names, this may contain the full name)
    pub first_name: String,
    /// Last name / Family name / 姓 (optional, may be empty for some name formats)
    pub last_name: Option<String>,
    pub affiliation: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Model {
    /// Get full name by combining first_name and last_name
    /// Returns "First Last" for Western names, "First" for names without last_name
    pub fn full_name(&self) -> String {
        match &self.last_name {
            Some(last) if !last.is_empty() => format!("{} {}", self.first_name, last),
            _ => self.first_name.clone(),
        }
    }

    /// Get display name in "Last, First" format (common for citations)
    /// Returns "Last, First" if last_name exists, otherwise just "First"
    pub fn citation_name(&self) -> String {
        match &self.last_name {
            Some(last) if !last.is_empty() => format!("{}, {}", last, self.first_name),
            _ => self.first_name.clone(),
        }
    }
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match *self {}
    }
}

impl ActiveModelBehavior for ActiveModel {}
