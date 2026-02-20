use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::sys::config::LlmProvider;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("No response from LLM")]
    NoResponse,

    #[error("Invalid API key")]
    InvalidApiKey,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: Option<String>,
}

pub struct LlmClient {
    client: Client,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Send a chat request to the LLM provider
    pub async fn chat(
        &self,
        provider: &LlmProvider,
        system_prompt: &str,
        user_content: &str,
    ) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", provider.base_url.trim_end_matches('/'));

        let request = ChatRequest {
            model: provider.model_name.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_content.to_string(),
                },
            ],
            temperature: 0.3,
        };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", provider.api_key))
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            // Try to parse error message
            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&body) {
                if let Some(error) = error_response.error {
                    if let Some(message) = error.message {
                        return Err(LlmError::ApiError(message));
                    }
                }
            }
            return Err(LlmError::ApiError(format!(
                "API returned status {}: {}",
                status, body
            )));
        }

        // Parse successful response
        let chat_response: ChatResponse =
            serde_json::from_str(&body).map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or(LlmError::NoResponse)?;

        Ok(content)
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}
