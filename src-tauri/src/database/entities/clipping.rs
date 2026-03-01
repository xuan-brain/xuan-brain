//! Clipping entity definition

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "clipping")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub url: String,
    pub content: Option<String>,
    pub source_domain: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub excerpt: Option<String>,
    pub thumbnail_url: Option<String>,
    pub read_status: i32,
    pub notes: Option<String>,
    /// JSON array of tags
    #[serde(default)]
    pub tags: Option<String>,
    /// JSON array of image paths
    #[serde(default)]
    pub image_paths: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match *self {}
    }
}

impl ActiveModelBehavior for ActiveModel {}
