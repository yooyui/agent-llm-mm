use std::sync::Arc;

use agent_llm_mm::{
    adapters::model::openai_compatible::OpenAiCompatibleModel,
    domain::{
        self_revision::{SelfRevisionRequest, TriggerType},
        snapshot::SelfSnapshot,
        types::{EventKind, Namespace, Owner},
    },
    ports::{EvidenceQuery, ModelDecisionRequest, ModelPort},
    support::config::OpenAiCompatibleConfig,
};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::oneshot,
};

#[tokio::test]
async fn openai_compatible_model_parses_first_assistant_message_into_action() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "id": "chatcmpl-test",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "summarize_memory_state"
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let decision = model
        .decide(test_support::sample_request())
        .await
        .expect("decision");

    assert_eq!(decision.action, "summarize_memory_state");
    assert_eq!(
        stub.last_request_path().await.as_deref(),
        Some("/chat/completions")
    );
}

#[tokio::test]
async fn openai_compatible_model_rejects_empty_action() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "   "
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let error = model
        .decide(test_support::sample_request())
        .await
        .expect_err("empty content should fail");

    assert!(error.to_string().contains("empty model action"));
}

#[tokio::test]
async fn openai_compatible_model_surfaces_non_success_status() {
    let stub = test_support::StubServer::spawn(
        503,
        json!({
            "error": {
                "message": "temporary outage"
            }
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let error = model
        .decide(test_support::sample_request())
        .await
        .expect_err("non success status should fail");

    assert!(
        error
            .to_string()
            .contains("openai-compatible request failed")
    );
}

#[tokio::test]
async fn openai_compatible_model_parses_self_revision_proposal_from_assistant_message() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "{\"should_reflect\":true,\"rationale\":\"conflict-backed identity patch\",\"machine_patch\":{\"identity_patch\":{\"canonical_claims\":[\"identity:self=mentor\"]},\"commitment_patch\":null}}"
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let proposal = model
        .propose_self_revision(test_support::sample_self_revision_request())
        .await
        .expect("proposal");

    let request_json = stub.last_request_json().await.expect("request json");
    let user_prompt = request_json["messages"][1]["content"]
        .as_str()
        .expect("user prompt");

    assert!(proposal.should_reflect);
    assert_eq!(proposal.rationale, "conflict-backed identity patch");
    assert_eq!(
        proposal
            .machine_patch
            .identity_patch
            .expect("identity patch")
            .canonical_claims,
        vec!["identity:self=mentor".to_string()]
    );
    assert!(proposal.machine_patch.commitment_patch.is_none());
    assert_eq!(
        stub.last_request_path().await.as_deref(),
        Some("/chat/completions")
    );
    assert_eq!(request_json["model"], json!("gpt-4o-mini"));
    assert!(user_prompt.contains("\"trigger_type\": \"conflict\""));
    assert!(user_prompt.contains("\"evidence_event_ids\": ["));
    assert!(user_prompt.contains("\"evt-1\""));
    assert!(user_prompt.contains("\"trigger_hints\": ["));
    assert!(user_prompt.contains("\"claim conflict\""));
}

#[tokio::test]
async fn openai_compatible_model_defaults_missing_machine_patch_in_self_revision_proposal() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "{\"should_reflect\":false,\"rationale\":\"not enough evidence yet\"}"
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let proposal = model
        .propose_self_revision(test_support::sample_self_revision_request())
        .await
        .expect("proposal");

    assert!(!proposal.should_reflect);
    assert_eq!(proposal.rationale, "not enough evidence yet");
    assert!(proposal.machine_patch.identity_patch.is_none());
    assert!(proposal.machine_patch.commitment_patch.is_none());
}

#[tokio::test]
async fn openai_compatible_model_accepts_fenced_json_self_revision_proposal() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Here is the proposal:\n```json\n{\"should_reflect\":true,\"rationale\":\"periodic review found stable evidence\",\"machine_patch\":{\"identity_patch\":null,\"commitment_patch\":null}}\n```"
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let proposal = model
        .propose_self_revision(test_support::sample_self_revision_request())
        .await
        .expect("proposal");

    assert!(proposal.should_reflect);
    assert_eq!(proposal.rationale, "periodic review found stable evidence");
    assert!(proposal.machine_patch.identity_patch.is_none());
    assert!(proposal.machine_patch.commitment_patch.is_none());
}

#[tokio::test]
async fn openai_compatible_model_parses_self_revision_evidence_policy() {
    let stub = test_support::StubServer::spawn(
        200,
        json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "{\"should_reflect\":true,\"rationale\":\"rollback evidence supports a narrower commitment update\",\"machine_patch\":{\"identity_patch\":{\"canonical_claims\":[\"identity:self=architect\"]},\"commitment_patch\":null},\"proposed_evidence_event_ids\":[\"evt-1\",\"evt-2\"],\"proposed_evidence_query\":{\"owner\":\"Self_\",\"kind\":\"Action\",\"limit\":2},\"confidence\":\"medium\"}"
                }
            }]
        }),
    )
    .await;
    let model = OpenAiCompatibleModel::new(OpenAiCompatibleConfig {
        base_url: stub.base_url(),
        api_key: "example-test-key".to_string(),
        model: "gpt-4o-mini".to_string(),
        timeout_ms: 30_000,
    })
    .expect("model");

    let proposal = model
        .propose_self_revision(test_support::sample_self_revision_request())
        .await
        .expect("proposal");

    assert!(proposal.should_reflect);
    assert_eq!(
        proposal.proposed_evidence_event_ids,
        vec!["evt-1".to_string(), "evt-2".to_string()]
    );
    assert_eq!(
        proposal.proposed_evidence_query,
        Some(EvidenceQuery {
            namespace: None,
            owner: Some(Owner::Self_),
            kind: Some(EventKind::Action),
            limit: Some(2),
        })
    );
    assert_eq!(proposal.confidence.as_deref(), Some("medium"));
}

mod test_support {
    use super::*;

    pub fn sample_request() -> ModelDecisionRequest {
        ModelDecisionRequest::new(
            "summarize current memory".to_string(),
            "read_identity_core".to_string(),
            SelfSnapshot {
                identity: vec!["identity:self=architect".to_string()],
                commitments: Vec::new(),
                claims: vec!["self.role is architect".to_string()],
                evidence: vec!["event:evt-1".to_string()],
                episodes: vec!["episode:task-6".to_string()],
            },
        )
    }

    pub fn sample_self_revision_request() -> SelfRevisionRequest {
        SelfRevisionRequest::new(
            TriggerType::Conflict,
            Namespace::self_(),
            SelfSnapshot {
                identity: vec!["identity:self=architect".to_string()],
                commitments: vec!["forbid:write_identity_core_directly".to_string()],
                claims: vec!["self.role is architect".to_string()],
                evidence: vec!["event:evt-1".to_string()],
                episodes: vec!["episode:task-6".to_string()],
            },
            vec!["evt-1".to_string()],
            vec!["claim conflict".to_string()],
        )
    }

    pub struct StubServer {
        base_url: String,
        last_request_path: Arc<tokio::sync::Mutex<Option<String>>>,
        last_request_body: Arc<tokio::sync::Mutex<Option<String>>>,
        shutdown: Option<oneshot::Sender<()>>,
    }

    impl StubServer {
        pub async fn spawn(status: u16, body: Value) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let address = listener.local_addr().expect("local addr");
            let base_url = format!("http://{address}");
            let last_request_path = Arc::new(tokio::sync::Mutex::new(None));
            let last_request_body = Arc::new(tokio::sync::Mutex::new(None));
            let request_path = Arc::clone(&last_request_path);
            let request_body = Arc::clone(&last_request_body);
            let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
            let response_body = body.to_string();

            tokio::spawn(async move {
                tokio::select! {
                    _ = &mut shutdown_rx => {}
                    accept = listener.accept() => {
                        if let Ok((mut stream, _)) = accept {
                            let mut buffer = vec![0_u8; 16 * 1024];
                            let bytes_read = stream.read(&mut buffer).await.expect("read");
                            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                            let path = request
                                .lines()
                                .next()
                                .and_then(|line| line.split_whitespace().nth(1))
                                .map(str::to_string);
                            let body = request
                                .split_once("\r\n\r\n")
                                .map(|(_, body)| body.to_string());
                            *request_path.lock().await = path;
                            *request_body.lock().await = body;

                            let status_text = match status {
                                200 => "OK",
                                503 => "Service Unavailable",
                                _ => "Test Status",
                            };
                            let response = format!(
                                "HTTP/1.1 {status} {status_text}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                                response_body.len(),
                                response_body
                            );
                            stream
                                .write_all(response.as_bytes())
                                .await
                                .expect("write");
                        }
                    }
                }
            });

            Self {
                base_url,
                last_request_path,
                last_request_body,
                shutdown: Some(shutdown_tx),
            }
        }

        pub fn base_url(&self) -> String {
            self.base_url.clone()
        }

        pub async fn last_request_path(&self) -> Option<String> {
            self.last_request_path.lock().await.clone()
        }

        pub async fn last_request_body(&self) -> Option<String> {
            self.last_request_body.lock().await.clone()
        }

        pub async fn last_request_json(&self) -> Option<Value> {
            let body = self.last_request_body().await?;
            serde_json::from_str(&body).ok()
        }
    }

    impl Drop for StubServer {
        fn drop(&mut self) {
            if let Some(shutdown) = self.shutdown.take() {
                let _ = shutdown.send(());
            }
        }
    }
}
