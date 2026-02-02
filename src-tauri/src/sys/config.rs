use crate::sys::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct LlmProvider {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub base_url: String,
    pub model_name: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SystemConfig {
    pub llm_providers: Vec<LlmProvider>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub system: SystemConfig,
}

impl AppConfig {
    pub fn load(config_dir: &str) -> Result<Self> {
        let path = PathBuf::from(config_dir).join("settings.json");
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            AppError::file_system(
                path.to_string_lossy().to_string(),
                format!("Failed to read config file: {}", e),
            )
        })?;

        serde_json::from_str(&content).map_err(|e| {
            AppError::config_error(
                "settings.json",
                format!("Failed to parse config file: {}", e),
            )
        })
    }

    pub fn save(&self, config_dir: &str) -> Result<()> {
        let path = PathBuf::from(config_dir).join("settings.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            AppError::config_error(
                "settings.json",
                format!("Failed to serialize config: {}", e),
            )
        })?;

        let mut file = fs::File::create(&path).map_err(|e| {
            AppError::file_system(
                path.to_string_lossy().to_string(),
                format!("Failed to create config file: {}", e),
            )
        })?;

        file.write_all(content.as_bytes()).map_err(|e| {
            AppError::file_system(
                path.to_string_lossy().to_string(),
                format!("Failed to write config file: {}", e),
            )
        })?;

        Ok(())
    }
}
