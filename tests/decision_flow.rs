use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use agent_llm_mm::{
    application::decide_with_snapshot::{DecideWithSnapshotInput, execute},
    domain::snapshot::SelfSnapshot,
    error::AppError,
    ports::{ModelDecision, ModelDecisionRequest, ModelPort},
};
use async_trait::async_trait;

#[tokio::test]
async fn decision_returns_blocked_without_calling_model_when_gate_fails() {
    let deps = test_support::deps_with_blocking_commitment();

    let result = execute(&deps, test_support::blocked_decision_input())
        .await
        .unwrap();

    assert!(result.blocked);
    assert!(result.decision.is_none());
    assert_eq!(deps.model_call_count(), 0);
    assert!(deps.last_request().is_none());
}

#[tokio::test]
async fn mock_model_receives_snapshot_context_when_gate_passes() {
    let deps = test_support::deps_with_mock_model();
    let input = test_support::decision_input();

    let result = execute(&deps, input.clone()).await.unwrap();

    assert_eq!(
        result.decision,
        Some(ModelDecision::new("summarize_memory_state".to_string()))
    );

    let request = deps.last_request().expect("model should receive request");
    assert_eq!(request.task, input.task);
    assert_eq!(request.action, input.action);
    assert_eq!(request.snapshot, input.snapshot);
}

mod test_support {
    use super::*;

    use agent_llm_mm::adapters::model::mock::MockModel;

    pub fn deps_with_blocking_commitment() -> DecisionDeps {
        DecisionDeps {
            model: Arc::new(MockModel),
            model_calls: Arc::new(AtomicUsize::new(0)),
            last_request: Arc::new(Mutex::new(None)),
            snapshot: SelfSnapshot {
                identity: vec!["identity:self=architect".to_string()],
                commitments: vec!["forbid:write_identity_core_directly".to_string()],
                claims: vec!["self.role is architect".to_string()],
                evidence: vec!["event:evt-1".to_string()],
                episodes: vec!["episode:task-6".to_string()],
            },
        }
    }

    pub fn deps_with_mock_model() -> DecisionDeps {
        DecisionDeps {
            model: Arc::new(MockModel),
            model_calls: Arc::new(AtomicUsize::new(0)),
            last_request: Arc::new(Mutex::new(None)),
            snapshot: SelfSnapshot {
                identity: vec!["identity:self=architect".to_string()],
                commitments: Vec::new(),
                claims: vec!["self.role is architect".to_string()],
                evidence: vec!["event:evt-1".to_string()],
                episodes: vec!["episode:task-6".to_string()],
            },
        }
    }

    pub fn blocked_decision_input() -> DecideWithSnapshotInput {
        DecideWithSnapshotInput {
            task: "summarize current memory".to_string(),
            action: "write_identity_core_directly".to_string(),
            snapshot: deps_with_blocking_commitment().snapshot,
        }
    }

    pub fn decision_input() -> DecideWithSnapshotInput {
        DecideWithSnapshotInput {
            task: "summarize current memory".to_string(),
            action: "read_identity_core".to_string(),
            snapshot: deps_with_mock_model().snapshot,
        }
    }

    #[derive(Clone)]
    pub struct DecisionDeps {
        pub model: Arc<MockModel>,
        pub model_calls: Arc<AtomicUsize>,
        pub last_request: Arc<Mutex<Option<ModelDecisionRequest>>>,
        pub snapshot: SelfSnapshot,
    }

    impl DecisionDeps {
        pub fn model_call_count(&self) -> usize {
            self.model_calls.load(Ordering::SeqCst)
        }

        pub fn last_request(&self) -> Option<ModelDecisionRequest> {
            self.last_request.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ModelPort for DecisionDeps {
        async fn decide(&self, request: ModelDecisionRequest) -> Result<ModelDecision, AppError> {
            self.model_calls.fetch_add(1, Ordering::SeqCst);
            *self.last_request.lock().unwrap() = Some(request.clone());
            self.model.decide(request).await
        }
    }
}
