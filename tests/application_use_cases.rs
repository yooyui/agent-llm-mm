use std::sync::{Arc, Mutex};

use agent_llm_mm::{
    application::{
        ingest_interaction::{IngestInput, execute as execute_ingest},
        run_reflection::{ReflectionInput, execute as execute_reflection},
    },
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        types::{EventKind, Mode, Owner},
    },
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EventStore, IdGenerator, IdentityStore,
        ModelDecision, ModelInput, ModelPort, ReflectionStore, StoredClaim, StoredEvent,
        StoredReflection,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[tokio::test]
async fn ingest_writes_events_before_claims() {
    let deps = test_support::in_memory_deps();

    let result = execute_ingest(&deps, test_support::ingest_input()).await;

    assert!(result.is_ok());
    assert_eq!(
        deps.log(),
        vec!["append_event", "upsert_claim", "link_evidence"]
    );
}

#[tokio::test]
async fn reflection_supersedes_old_claim_instead_of_deleting_it() {
    let deps = test_support::in_memory_deps();

    let result = execute_reflection(&deps, test_support::reflection_input()).await;

    assert!(result.is_ok());
    assert!(deps.was_status_updated_to("superseded"));
}

mod test_support {
    use super::*;

    pub fn in_memory_deps() -> InMemoryDeps {
        InMemoryDeps {
            state: Arc::new(Mutex::new(State {
                log: Vec::new(),
                next_id: 1,
                now: chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                statuses: Vec::new(),
                commitments: vec![Commitment::new(
                    Owner::Self_,
                    "forbid:write_identity_core_directly",
                )],
                identity: IdentityCore::new(vec!["identity:self=architect".to_string()]),
            })),
        }
    }

    pub fn ingest_input() -> IngestInput {
        IngestInput::new(
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "The user asked for stronger memory.",
            ),
            vec![ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "architect",
                Mode::Observed,
            )],
            Some("episode:task-4".to_string()),
        )
    }

    pub fn reflection_input() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("The previous role claim needs to be superseded by a newer version."),
            "claim-old",
            Some(ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "senior_architect",
                Mode::Inferred,
            )),
        )
    }
}

#[derive(Clone)]
struct InMemoryDeps {
    state: Arc<Mutex<State>>,
}

struct State {
    log: Vec<String>,
    next_id: usize,
    now: DateTime<Utc>,
    statuses: Vec<ClaimStatus>,
    commitments: Vec<Commitment>,
    identity: IdentityCore,
}

impl InMemoryDeps {
    fn log(&self) -> Vec<&'static str> {
        self.state
            .lock()
            .unwrap()
            .log
            .iter()
            .map(|entry| match entry.as_str() {
                "append_event" => "append_event",
                "upsert_claim" => "upsert_claim",
                "link_evidence" => "link_evidence",
                _ => "other",
            })
            .collect()
    }

    fn was_status_updated_to(&self, expected: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .statuses
            .iter()
            .any(|status| status.as_str() == expected)
    }
}

#[async_trait]
impl EventStore for InMemoryDeps {
    async fn append_event(&self, _event: StoredEvent) -> Result<(), agent_llm_mm::error::AppError> {
        self.state
            .lock()
            .unwrap()
            .log
            .push("append_event".to_string());
        Ok(())
    }

    async fn list_event_references(&self) -> Result<Vec<String>, agent_llm_mm::error::AppError> {
        Ok(vec!["event:evt-1".to_string()])
    }
}

#[async_trait]
impl ClaimStore for InMemoryDeps {
    async fn upsert_claim(&self, _claim: StoredClaim) -> Result<(), agent_llm_mm::error::AppError> {
        self.state
            .lock()
            .unwrap()
            .log
            .push("upsert_claim".to_string());
        Ok(())
    }

    async fn link_evidence(
        &self,
        _claim_id: String,
        _event_id: String,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        self.state
            .lock()
            .unwrap()
            .log
            .push("link_evidence".to_string());
        Ok(())
    }

    async fn list_active_claims(&self) -> Result<Vec<StoredClaim>, agent_llm_mm::error::AppError> {
        Ok(vec![StoredClaim::new(
            "claim-active".to_string(),
            ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
            ClaimStatus::Active,
        )])
    }

    async fn update_claim_status(
        &self,
        _claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        self.state.lock().unwrap().statuses.push(status);
        Ok(())
    }
}

#[async_trait]
impl IdentityStore for InMemoryDeps {
    async fn load_identity(&self) -> Result<IdentityCore, agent_llm_mm::error::AppError> {
        Ok(self.state.lock().unwrap().identity.clone())
    }

    async fn save_identity(
        &self,
        _identity: IdentityCore,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for InMemoryDeps {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, agent_llm_mm::error::AppError> {
        Ok(self.state.lock().unwrap().commitments.clone())
    }
}

#[async_trait]
impl agent_llm_mm::ports::EpisodeStore for InMemoryDeps {
    async fn record_event_in_episode(
        &self,
        _episode_reference: String,
        _event_id: String,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        Ok(())
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, agent_llm_mm::error::AppError> {
        Ok(vec!["episode:task-4".to_string()])
    }
}

#[async_trait]
impl ReflectionStore for InMemoryDeps {
    async fn append_reflection(
        &self,
        _reflection: StoredReflection,
    ) -> Result<(), agent_llm_mm::error::AppError> {
        Ok(())
    }
}

#[async_trait]
impl ModelPort for InMemoryDeps {
    async fn decide(
        &self,
        _input: ModelInput,
    ) -> Result<ModelDecision, agent_llm_mm::error::AppError> {
        Ok(ModelDecision::new("Proceed".to_string()))
    }
}

#[async_trait]
impl Clock for InMemoryDeps {
    async fn now(&self) -> Result<DateTime<Utc>, agent_llm_mm::error::AppError> {
        Ok(self.state.lock().unwrap().now)
    }
}

#[async_trait]
impl IdGenerator for InMemoryDeps {
    async fn next_id(&self) -> Result<String, agent_llm_mm::error::AppError> {
        let mut state = self.state.lock().unwrap();
        let id = format!("id-{}", state.next_id);
        state.next_id += 1;
        Ok(id)
    }
}
