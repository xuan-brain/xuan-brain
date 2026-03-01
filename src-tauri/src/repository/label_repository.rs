//! Label repository for SQLite using SeaORM

use sea_orm::*;
use tracing::info;

use crate::database::entities::{label, paper_label};
use crate::models::{CreateLabel, Label, UpdateLabel};
use crate::sys::error::{AppError, Result};

/// Repository for Label operations
pub struct LabelRepository;

impl LabelRepository {
    /// Find all labels
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Label>> {
        let labels = label::Entity::find()
            .order_by_asc(label::Column::Name)
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query labels: {}", e)))?;

        info!("Found {} labels", labels.len());
        Ok(labels.into_iter().map(Label::from).collect())
    }

    /// Find label by ID
    pub async fn find_by_id(db: &DatabaseConnection, id: i64) -> Result<Option<Label>> {
        let label = label::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get label: {}", e)))?;

        Ok(label.map(Label::from))
    }

    /// Find label by name
    pub async fn find_by_name(db: &DatabaseConnection, name: &str) -> Result<Option<Label>> {
        let label = label::Entity::find()
            .filter(label::Column::Name.eq(name))
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to query label by name: {}", e)))?;

        Ok(label.map(Label::from))
    }

    /// Create a new label
    pub async fn create(db: &DatabaseConnection, create: CreateLabel) -> Result<Label> {
        // Check if label with same name already exists
        if Self::find_by_name(db, &create.name).await?.is_some() {
            return Err(AppError::validation(
                "name",
                format!("Label with name '{}' already exists", create.name),
            ));
        }

        let now = chrono::Utc::now();
        let new_label = label::ActiveModel {
            name: Set(create.name),
            color: Set(create.color),
            document_count: Set(0),
            created_at: Set(now),
            ..Default::default()
        };

        let result = new_label
            .insert(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to create label: {}", e)))?;

        Ok(Label::from(result))
    }

    /// Update label
    pub async fn update(db: &DatabaseConnection, id: i64, update: UpdateLabel) -> Result<Label> {
        // Check if another label with same name exists
        if let Some(ref name) = update.name {
            if let Some(existing) = Self::find_by_name(db, name).await? {
                if existing.id != id {
                    return Err(AppError::validation(
                        "name",
                        format!("Label with name '{}' already exists", name),
                    ));
                }
            }
        }

        let label = label::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find label: {}", e)))?
            .ok_or_else(|| AppError::not_found("Label", id.to_string()))?;

        let mut label: label::ActiveModel = label.into();
        if let Some(name) = update.name {
            label.name = Set(name);
        }
        if let Some(color) = update.color {
            label.color = Set(color);
        }

        let result = label
            .update(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to update label: {}", e)))?;

        Ok(Label::from(result))
    }

    /// Delete label
    pub async fn delete(db: &DatabaseConnection, id: i64) -> Result<()> {
        // First delete all paper-label relations (cascade will handle this, but we do it explicitly for safety)
        paper_label::Entity::delete_many()
            .filter(paper_label::Column::LabelId.eq(id))
            .exec(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to delete label relations: {}", e))
            })?;

        // Then delete the label
        label::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete label: {}", e)))?;

        Ok(())
    }

    /// Add label to paper
    pub async fn add_to_paper(db: &DatabaseConnection, paper_id: i64, label_id: i64) -> Result<()> {
        // Check if relation already exists
        let existing = paper_label::Entity::find()
            .filter(paper_label::Column::PaperId.eq(paper_id))
            .filter(paper_label::Column::LabelId.eq(label_id))
            .one(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to check existing relation: {}", e))
            })?;

        if existing.is_none() {
            let relation = paper_label::ActiveModel {
                paper_id: Set(paper_id),
                label_id: Set(label_id),
                ..Default::default()
            };
            relation.insert(db).await.map_err(|e| {
                AppError::generic(format!("Failed to add label to paper: {}", e))
            })?;
        }

        // Update document count
        Self::update_document_count(db, label_id).await?;

        Ok(())
    }

    /// Remove label from paper
    pub async fn remove_from_paper(
        db: &DatabaseConnection,
        paper_id: i64,
        label_id: i64,
    ) -> Result<()> {
        paper_label::Entity::delete_many()
            .filter(paper_label::Column::PaperId.eq(paper_id))
            .filter(paper_label::Column::LabelId.eq(label_id))
            .exec(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to remove label from paper: {}", e))
            })?;

        // Update document count
        Self::update_document_count(db, label_id).await?;

        Ok(())
    }

    /// Get labels for a paper
    pub async fn get_paper_labels(db: &DatabaseConnection, paper_id: i64) -> Result<Vec<Label>> {
        // First get paper_label relations
        let relations = paper_label::Entity::find()
            .filter(paper_label::Column::PaperId.eq(paper_id))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper-label relations: {}", e)))?;

        let label_ids: Vec<i64> = relations.iter().map(|r| r.label_id).collect();

        if label_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Then get labels by IDs
        let labels = label::Entity::find()
            .filter(label::Column::Id.is_in(label_ids))
            .all(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper labels: {}", e)))?;

        Ok(labels.into_iter().map(Label::from).collect())
    }

    /// Update document count for a label
    async fn update_document_count(db: &DatabaseConnection, label_id: i64) -> Result<()> {
        let count = paper_label::Entity::find()
            .filter(paper_label::Column::LabelId.eq(label_id))
            .count(db)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to count label documents: {}", e))
            })?;

        let label = label::Entity::find_by_id(label_id)
            .one(db)
            .await
            .map_err(|e| AppError::generic(format!("Failed to find label: {}", e)))?
            .ok_or_else(|| AppError::not_found("Label", label_id.to_string()))?;

        let mut label: label::ActiveModel = label.into();
        label.document_count = Set(count as i32);
        label.update(db).await.map_err(|e| {
            AppError::generic(format!("Failed to update label document count: {}", e))
        })?;

        Ok(())
    }
}
