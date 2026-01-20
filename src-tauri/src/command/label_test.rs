use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
use tauri::State;

use crate::database::entities::{label, prelude::Label};

#[tauri::command]
pub async fn init_test_labels(connection: State<'_, DatabaseConnection>) -> Result<String, String> {
    // Check if labels already exist
    let existing_labels = Label::find()
        .all(connection.inner())
        .await
        .map_err(|e| e.to_string())?;

    if !existing_labels.is_empty() {
        return Ok(format!("Already have {} labels", existing_labels.len()));
    }

    let test_labels = vec![
        ("AI", "#3b82f6"),
        ("Machine Learning", "#a855f7"),
        ("Deep Learning", "#ec4899"),
        ("NLP", "#ef4444"),
        ("Computer Vision", "#f97316"),
        ("Robotics", "#f59e0b"),
        ("Data Science", "#10b981"),
        ("Statistics", "#06b6d4"),
    ];

    let mut count = 0;
    for (name, color) in test_labels {
        let new_label = label::ActiveModel {
            name: Set(name.to_string()),
            color: Set(color.to_string()),
            ..Default::default()
        };

        new_label
            .insert(connection.inner())
            .await
            .map_err(|e| format!("Failed to insert label '{}': {}", name, e))?;
        count += 1;
    }

    Ok(format!("Created {} test labels", count))
}
