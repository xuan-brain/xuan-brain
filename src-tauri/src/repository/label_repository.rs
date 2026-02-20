//! Label repository for SurrealDB

use crate::surreal::connection::SurrealClient;
use crate::surreal::models::{CreateLabel, Label, UpdateLabel};
use crate::sys::error::{AppError, Result};
use surrealdb_types::RecordIdKey;
use tracing::info;

/// Convert RecordIdKey to string
fn record_id_key_to_string(key: &RecordIdKey) -> String {
    match key {
        RecordIdKey::String(s) => s.clone(),
        RecordIdKey::Number(n) => n.to_string(),
        RecordIdKey::Uuid(u) => u.to_string(),
        RecordIdKey::Array(_) => "array".to_string(),
        RecordIdKey::Object(_) => "object".to_string(),
        RecordIdKey::Range(_) => "range".to_string(),
    }
}

/// Repository for Label operations
pub struct LabelRepository<'a> {
    db: &'a SurrealClient,
}

impl<'a> LabelRepository<'a> {
    pub fn new(db: &'a SurrealClient) -> Self {
        Self { db }
    }

    /// Find all labels
    pub async fn find_all(&self) -> Result<Vec<Label>> {
        let result: Vec<Label> = self
            .db
            .query("SELECT * FROM label ORDER BY name")
            .await
            .map_err(|e| AppError::generic(format!("Failed to query labels: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        info!("Found {} labels", result.len());
        Ok(result)
    }

    /// Find label by ID (string format like "label:123")
    pub async fn find_by_id(&self, id: &str) -> Result<Option<Label>> {
        let id = id.to_string();
        let result: Vec<Label> = self
            .db
            .query("SELECT * FROM type::thing($id) LIMIT 1")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get label: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Find label by name
    pub async fn find_by_name(&self, name: &str) -> Result<Option<Label>> {
        let name = name.to_string();
        let result: Vec<Label> = self
            .db
            .query("SELECT * FROM label WHERE name = $name LIMIT 1")
            .bind(("name", name))
            .await
            .map_err(|e| AppError::generic(format!("Failed to query label by name: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result.into_iter().next())
    }

    /// Create a new label
    pub async fn create(&self, label: CreateLabel) -> Result<Label> {
        // Check if label with same name already exists
        if (self.find_by_name(&label.name).await?).is_some() {
            return Err(AppError::validation(
                "name",
                format!("Label with name '{}' already exists", label.name),
            ));
        }

        let label = Label::from(label);

        let result: Vec<Label> = self
            .db
            .query("CREATE label CONTENT $label")
            .bind(("label", label))
            .await
            .map_err(|e| AppError::generic(format!("Failed to create label: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::generic("Failed to create label".to_string()))
    }

    /// Update label
    pub async fn update(&self, id: &str, update: UpdateLabel) -> Result<Label> {
        let id_owned = id.to_string();

        if let Some(ref name) = update.name {
            // Check if another label with same name exists
            if let Some(existing) = self.find_by_name(name).await? {
                let existing_id_str = existing.id.as_ref().map(|rid| format!("{}:{}", rid.table, record_id_key_to_string(&rid.key)));
                if existing_id_str.as_deref() != Some(id) {
                    return Err(AppError::validation(
                        "name",
                        format!("Label with name '{}' already exists", name),
                    ));
                }
            }
        }

        let mut sets = Vec::new();
        if update.name.is_some() {
            sets.push("name = $name");
        }
        if update.color.is_some() {
            sets.push("color = $color");
        }

        if sets.is_empty() {
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::not_found("Label", id_owned));
        }

        let query = format!("UPDATE type::thing($id) SET {}", sets.join(", "));

        let result: Vec<Label> = self
            .db
            .query(&query)
            .bind(("id", id_owned.clone()))
            .bind(("name", update.name))
            .bind(("color", update.color))
            .await
            .map_err(|e| AppError::generic(format!("Failed to update label: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        result
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("Label", id_owned))
    }

    /// Delete label
    pub async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        // First delete all paper-label relations
        self.db
            .query("DELETE paper_label WHERE `out` = type::thing($id)")
            .bind(("id", id.clone()))
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to delete label relations: {}", e))
            })?;

        // Then delete the label
        self.db
            .query("DELETE type::thing($id)")
            .bind(("id", id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to delete label: {}", e)))?;

        Ok(())
    }

    /// Add label to paper
    pub async fn add_to_paper(&self, paper_id: &str, label_id: &str) -> Result<()> {
        let paper_id = paper_id.to_string();
        let label_id = label_id.to_string();

        // Use inline record ID format for RELATE statement
        // SurrealDB 3.0 requires direct record ID syntax in RELATE
        let query = format!(
            "RELATE {}->paper_label->{}",
            paper_id, label_id
        );
        self.db
            .query(&query)
            .await
            .map_err(|e| AppError::generic(format!("Failed to add label to paper: {}", e)))?;

        // Update document count
        self.db
            .query(
                r#"
                UPDATE type::thing($label) SET document_count = array::len(
                    SELECT VALUE `in` FROM paper_label WHERE `out` = type::thing($label)
                )
                "#,
            )
            .bind(("label", label_id))
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to update label document count: {}", e))
            })?;

        Ok(())
    }

    /// Remove label from paper
    pub async fn remove_from_paper(&self, paper_id: &str, label_id: &str) -> Result<()> {
        let paper_id = paper_id.to_string();
        let label_id = label_id.to_string();

        // Use inline record ID format for DELETE statement with WHERE clause
        let query = format!(
            "DELETE paper_label WHERE `in` = {} AND `out` = {}",
            paper_id, label_id
        );
        self.db
            .query(&query)
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to remove label from paper: {}", e))
            })?;

        // Update document count
        self.db
            .query(
                r#"
                UPDATE type::thing($label) SET document_count = array::len(
                    SELECT VALUE `in` FROM paper_label WHERE `out` = type::thing($label)
                )
                "#,
            )
            .bind(("label", label_id))
            .await
            .map_err(|e| {
                AppError::generic(format!("Failed to update label document count: {}", e))
            })?;

        Ok(())
    }

    /// Get labels for a paper
    pub async fn get_paper_labels(&self, paper_id: &str) -> Result<Vec<Label>> {
        let paper_id = paper_id.to_string();
        let result: Vec<Label> = self
            .db
            .query(
                r#"
                SELECT * FROM label
                WHERE id IN (SELECT VALUE `out` FROM paper_label WHERE `in` = type::thing($paper))
                "#,
            )
            .bind(("paper", paper_id))
            .await
            .map_err(|e| AppError::generic(format!("Failed to get paper labels: {}", e)))?
            .take(0)
            .map_err(|e| AppError::generic(format!("Failed to get results: {}", e)))?;

        Ok(result)
    }
}
