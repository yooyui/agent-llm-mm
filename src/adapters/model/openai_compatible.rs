use async_trait::async_trait;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    domain::self_revision::{SelfRevisionProposal, SelfRevisionRequest},
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
        let body = self
            .send_chat_completion(build_decision_payload(&self.config.model, request))
            .await?;
        let action = extract_action(body)?;
        Ok(ModelDecision::new(action))
    }

    async fn propose_self_revision(
        &self,
        request: SelfRevisionRequest,
    ) -> Result<SelfRevisionProposal, AppError> {
        let body = self
            .send_chat_completion(build_self_revision_payload(&self.config.model, request))
            .await?;
        extract_self_revision_proposal(body)
    }
}

impl OpenAiCompatibleModel {
    async fn send_chat_completion(
        &self,
        payload: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, AppError> {
        let response = self
            .client
            .post(self.endpoint())
            .bearer_auth(&self.config.api_key)
            .json(&payload)
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

        response.json().await.map_err(|error| {
            AppError::Message(format!(
                "openai-compatible response could not be parsed: {error}"
            ))
        })
    }
}

fn build_decision_payload(model: &str, request: ModelDecisionRequest) -> ChatCompletionRequest {
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

fn build_self_revision_payload(model: &str, request: SelfRevisionRequest) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: model.to_string(),
        temperature: 0.0,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "Return only a JSON self-revision proposal with should_reflect, rationale, machine_patch.identity_patch, machine_patch.commitment_patch, proposed_evidence_event_ids, proposed_evidence_query, and confidence.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Self revision request:\n{}",
                    serde_json::to_string_pretty(&request)
                        .unwrap_or_else(|_| "{}".to_string())
                ),
            },
        ],
    }
}

fn extract_action(response: ChatCompletionResponse) -> Result<String, AppError> {
    extract_message_content(
        response,
        "openai-compatible response contained an empty model action",
    )
}

fn extract_self_revision_proposal(
    response: ChatCompletionResponse,
) -> Result<SelfRevisionProposal, AppError> {
    let content = extract_message_content(
        response,
        "openai-compatible response contained an empty self-revision proposal",
    )?;
    let proposal_json = extract_json_object(&content).ok_or_else(|| {
        AppError::Message(
            "openai-compatible self-revision proposal did not contain a JSON object".to_string(),
        )
    })?;
    serde_json::from_str(proposal_json).map_err(|error| {
        AppError::Message(format!(
            "openai-compatible self-revision proposal could not be parsed: {error}"
        ))
    })
}

fn extract_message_content(
    response: ChatCompletionResponse,
    empty_message: &str,
) -> Result<String, AppError> {
    let content = response
        .choices
        .into_iter()
        .find_map(|choice| choice.message.content)
        .map(|content| content.trim().to_string())
        .unwrap_or_default();

    if content.is_empty() {
        return Err(AppError::Message(empty_message.to_string()));
    }

    Ok(content)
}

fn extract_json_object(content: &str) -> Option<&str> {
    let trimmed = content.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    let stripped_fence = strip_code_fence(trimmed).unwrap_or(trimmed);
    if stripped_fence.starts_with('{') && stripped_fence.ends_with('}') {
        return Some(stripped_fence);
    }

    let start = stripped_fence.find('{')?;
    let end = stripped_fence.rfind('}')?;
    (start < end).then_some(&stripped_fence[start..=end])
}

fn strip_code_fence(content: &str) -> Option<&str> {
    let fenced = content.strip_prefix("```")?;
    let fenced = match fenced.find('\n') {
        Some(index) => &fenced[index + 1..],
        None => return None,
    };
    fenced.strip_suffix("```").map(str::trim)
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
