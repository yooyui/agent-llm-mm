use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    error::AppError,
    ports::{ModelDecision, ModelDecisionRequest, ModelPort},
    support::config::OpenAiCompatibleConfig,
};

#[derive(Debug, Clone)]
pub struct OpenAiCompatibleModel {
    client: reqwest::Client,
    config: OpenAiCompatibleConfig,
}

impl OpenAiCompatibleModel {
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self, AppError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|error| AppError::Message(format!("failed to build http client: {error}")))?;

        Ok(Self { client, config })
    }

    fn endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        )
    }
}

#[async_trait]
impl ModelPort for OpenAiCompatibleModel {
    async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
        let response = self
            .client
            .post(self.endpoint())
            .bearer_auth(&self.config.api_key)
            .json(&build_payload(&self.config.model, request))
            .send()
            .await
            .map_err(|error| {
                AppError::Message(format!("openai-compatible network error: {error}"))
            })?;

        let status = response.status();
        if status != StatusCode::OK {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Message(format!(
                "openai-compatible request failed with status {}: {}",
                status.as_u16(),
                body
            )));
        }

        let body: ChatCompletionResponse = response.json().await.map_err(|error| {
            AppError::Message(format!(
                "openai-compatible response could not be parsed: {error}"
            ))
        })?;

        let action = extract_action(body)?;
        Ok(ModelDecision::new(action))
    }
}

fn build_payload(model: &str, request: ModelDecisionRequest) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: model.to_string(),
        temperature: 0.0,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Return only the next action name as plain text with no explanation."
                    .to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Task: {}\nAction: {}\nSnapshot:\n{}",
                    request.task,
                    request.action,
                    serde_json::to_string_pretty(&request.snapshot)
                        .unwrap_or_else(|_| "{}".to_string())
                ),
            },
        ],
    }
}

fn extract_action(response: ChatCompletionResponse) -> Result<String, AppError> {
    let action = response
        .choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .unwrap_or_default();

    if action.is_empty() {
        return Err(AppError::Message(
            "openai-compatible response contained an empty model action".to_string(),
        ));
    }

    Ok(action)
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}
