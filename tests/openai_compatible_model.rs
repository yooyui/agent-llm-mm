use std::sync::Arc;

use agent_llm_mm::{
    adapters::model::openai_compatible::OpenAiCompatibleModel,
    domain::snapshot::SelfSnapshot,
    ports::{ModelDecisionRequest, ModelPort},
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

    pub struct StubServer {
        base_url: String,
        last_request_path: Arc<tokio::sync::Mutex<Option<String>>>,
        shutdown: Option<oneshot::Sender<()>>,
    }

    impl StubServer {
        pub async fn spawn(status: u16, body: Value) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
            let address = listener.local_addr().expect("local addr");
            let base_url = format!("http://{address}");
            let last_request_path = Arc::new(tokio::sync::Mutex::new(None));
            let request_path = Arc::clone(&last_request_path);
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
                            *request_path.lock().await = path;

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
                shutdown: Some(shutdown_tx),
            }
        }

        pub fn base_url(&self) -> String {
            self.base_url.clone()
        }

        pub async fn last_request_path(&self) -> Option<String> {
            self.last_request_path.lock().await.clone()
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
