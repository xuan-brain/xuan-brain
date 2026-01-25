use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use serde::Serialize;
use tauri::State;

use crate::database::entities::{papers, prelude::*};

#[derive(Serialize)]
pub struct PaperDto {
    pub id: i64,
    pub title: String,
    pub publication_year: Option<i64>,
    pub journal_name: Option<String>,
    pub conference_name: Option<String>,
    pub authors: Vec<String>,
}

#[tauri::command]
pub async fn get_all_papers(db: State<'_, DatabaseConnection>) -> Result<Vec<PaperDto>, String> {
    let papers = Papers::find()
        .find_with_related(Authors)
        .order_by_desc(papers::Column::Id)
        .all(db.inner())
        .await
        .map_err(|e| e.to_string())?;

    let dtos: Vec<PaperDto> = papers
        .into_iter()
        .map(|(paper, authors)| PaperDto {
            id: paper.id,
            title: paper.title,
            publication_year: paper.publication_year,
            journal_name: paper.journal_name,
            conference_name: paper.conference_name,
            authors: authors.into_iter().map(|a| a.name).collect(),
        })
        .collect();

    Ok(dtos)
}
