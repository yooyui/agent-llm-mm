use std::sync::{Arc, Mutex};

use agent_llm_mm::{
    application::{
        build_self_snapshot::{BuildSelfSnapshotInput, execute as execute_build_snapshot},
        ingest_interaction::{IngestInput, execute as execute_ingest},
        run_reflection::{ReflectionInput, execute as execute_reflection},
    },
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        snapshot::SnapshotBudget,
        types::{EventKind, Mode, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, IdGenerator,
        IdentityStore, IngestTransaction, IngestTransactionRunner, ReflectionTransaction,
        ReflectionTransactionRunner, StoredClaim, StoredEvent, StoredReflection,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[tokio::test]
async fn inferred_claim_without_evidence_is_rejected() {
    let deps = test_support::deps_for_failure_modes();

    let result = execute_ingest(&deps, test_support::inferred_without_evidence()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn reflection_marks_conflict_as_disputed_instead_of_deleting_history() {
    let deps = test_support::deps_for_failure_modes();

    execute_reflection(&deps, test_support::conflicting_reflection())
        .await
        .unwrap();

    assert!(deps.history_contains_status("disputed"));
}

#[tokio::test]
async fn snapshot_budget_prevents_recent_event_hijack() {
    let deps = test_support::deps_for_failure_modes();

    let snapshot = execute_build_snapshot(&deps, test_support::budgeted_snapshot())
        .await
        .unwrap()
        .snapshot;

    assert!(snapshot.evidence.len() <= 3);
    assert!(
        snapshot.evidence.contains(&"event:anchor".to_string()),
        "stable evidence should survive budgeting: {:?}",
        snapshot.evidence
    );
    assert_eq!(
        snapshot
            .evidence
            .iter()
            .filter(|reference| reference.as_str() == "event:recent-noise")
            .count(),
        1,
        "duplicate recent evidence should not consume the whole budget: {:?}",
        snapshot.evidence
    );
}

mod test_support {
    use super::*;

    pub fn deps_for_failure_modes() -> FailureModeDeps {
        FailureModeDeps::new(State::default())
    }

    pub fn inferred_without_evidence() -> IngestInput {
        IngestInput::new(
            Event::new(
                Owner::User,
                EventKind::Conversation,
                "The user guessed at an unsupported identity change.",
            ),
            vec![ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "principal_architect",
                Mode::Inferred,
            )],
            None,
        )
    }

    pub fn conflicting_reflection() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("A conflicting reflection should mark the claim as disputed."),
            "claim-conflict",
            None,
        )
    }

    pub fn budgeted_snapshot() -> BuildSelfSnapshotInput {
        BuildSelfSnapshotInput {
            budget: SnapshotBudget::new(3),
        }
    }
}

#[derive(Clone)]
struct FailureModeDeps {
    state: Arc<Mutex<State>>,
}

#[derive(Clone)]
struct State {
    committed: CommittedState,
    now: DateTime<Utc>,
    next_id: usize,
}

#[derive(Clone)]
struct CommittedState {
    claims: Vec<StoredClaim>,
    commitments: Vec<Commitment>,
    identity: IdentityCore,
    event_references: Vec<String>,
    episode_references: Vec<String>,
    reflections: Vec<StoredReflection>,
    evidence_links: Vec<(String, String)>,
    events: Vec<StoredEvent>,
}

#[derive(Default)]
struct PendingIngest {
    claims: Vec<StoredClaim>,
    evidence_links: Vec<(String, String)>,
    events: Vec<StoredEvent>,
}

#[derive(Default)]
struct PendingReflection {
    claims: Vec<StoredClaim>,
    reflections: Vec<StoredReflection>,
    status_updates: Vec<(String, ClaimStatus)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            committed: CommittedState {
                claims: vec![StoredClaim::new(
                    "claim-conflict".to_string(),
                    ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                    ClaimStatus::Active,
                )],
                commitments: vec![Commitment::new(
                    Owner::Self_,
                    "forbid:write_identity_core_directly",
                )],
                identity: IdentityCore::new(vec!["identity:self=architect".to_string()]),
                event_references: vec![
                    "event:recent-noise".to_string(),
                    "event:recent-noise".to_string(),
                    "event:recent-noise".to_string(),
                    "event:anchor".to_string(),
                    "event:baseline".to_string(),
                ],
                episode_references: vec!["episode:failure-modes".to_string()],
                reflections: Vec::new(),
                evidence_links: Vec::new(),
                events: Vec::new(),
            },
            now: chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            next_id: 1,
        }
    }
}

impl FailureModeDeps {
    fn new(state: State) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    fn history_contains_status(&self, status: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .committed
            .claims
            .iter()
            .any(|claim| claim.status.as_str() == status)
    }
}

fn upsert_claim(target: &mut Vec<StoredClaim>, claim: StoredClaim) {
    if let Some(existing) = target
        .iter_mut()
        .find(|stored_claim| stored_claim.claim_id == claim.claim_id)
    {
        *existing = claim;
    } else {
        target.push(claim);
    }
}

#[async_trait]
impl Clock for FailureModeDeps {
    async fn now(&self) -> Result<DateTime<Utc>, AppError> {
        Ok(self.state.lock().unwrap().now)
    }
}

#[async_trait]
impl IdGenerator for FailureModeDeps {
    async fn next_id(&self) -> Result<String, AppError> {
        let mut state = self.state.lock().unwrap();
        let next_id = format!("id-{}", state.next_id);
        state.next_id += 1;
        Ok(next_id)
    }
}

#[async_trait]
impl IdentityStore for FailureModeDeps {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        Ok(self.state.lock().unwrap().committed.identity.clone())
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        self.state.lock().unwrap().committed.identity = identity;
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for FailureModeDeps {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError> {
        Ok(self.state.lock().unwrap().committed.commitments.clone())
    }
}

#[async_trait]
impl EventStore for FailureModeDeps {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError> {
        self.state.lock().unwrap().committed.events.push(event);
        Ok(())
    }

    async fn list_event_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .event_references
            .clone())
    }
}

#[async_trait]
impl EpisodeStore for FailureModeDeps {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        _event_id: String,
    ) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .episode_references
            .push(episode_reference);
        Ok(())
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .episode_references
            .clone())
    }
}

#[async_trait]
impl ClaimStore for FailureModeDeps {
    async fn upsert_claim(&self, claim: StoredClaim) -> Result<(), AppError> {
        upsert_claim(&mut self.state.lock().unwrap().committed.claims, claim);
        Ok(())
    }

    async fn link_evidence(&self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .evidence_links
            .push((claim_id, event_id));
        Ok(())
    }

    async fn list_active_claims(&self) -> Result<Vec<StoredClaim>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .claims
            .iter()
            .filter(|claim| claim.status == ClaimStatus::Active)
            .cloned()
            .collect())
    }

    async fn update_claim_status(
        &self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        let claim = state
            .committed
            .claims
            .iter_mut()
            .find(|claim| claim.claim_id == claim_id)
            .ok_or_else(|| AppError::Message(format!("missing claim: {claim_id}")))?;
        claim.status = status;
        Ok(())
    }
}

#[async_trait]
impl IngestTransactionRunner for FailureModeDeps {
    async fn begin_ingest_transaction(
        &self,
    ) -> Result<Box<dyn IngestTransaction + Send + '_>, AppError> {
        Ok(Box::new(FailureModeIngestTransaction {
            deps: self.clone(),
            pending: PendingIngest::default(),
        }))
    }
}

#[async_trait]
impl ReflectionTransactionRunner for FailureModeDeps {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + Send + '_>, AppError> {
        Ok(Box::new(FailureModeReflectionTransaction {
            deps: self.clone(),
            pending: PendingReflection::default(),
        }))
    }
}

struct FailureModeIngestTransaction {
    deps: FailureModeDeps,
    pending: PendingIngest,
}

#[async_trait]
impl IngestTransaction for FailureModeIngestTransaction {
    async fn append_event(&mut self, event: StoredEvent) -> Result<(), AppError> {
        self.pending.events.push(event);
        Ok(())
    }

    async fn record_event_in_episode(
        &mut self,
        _episode_reference: String,
        _event_id: String,
    ) -> Result<(), AppError> {
        Ok(())
    }

    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.pending.claims.push(claim);
        Ok(())
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.pending.evidence_links.push((claim_id, event_id));
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        let mut state = self.deps.state.lock().unwrap();
        state.committed.events.extend(self.pending.events);
        state
            .committed
            .evidence_links
            .extend(self.pending.evidence_links);
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        Ok(())
    }
}

struct FailureModeReflectionTransaction {
    deps: FailureModeDeps,
    pending: PendingReflection,
}

#[async_trait]
impl ReflectionTransaction for FailureModeReflectionTransaction {
    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.pending.claims.push(claim);
        Ok(())
    }

    async fn append_reflection(&mut self, reflection: StoredReflection) -> Result<(), AppError> {
        self.pending.reflections.push(reflection);
        Ok(())
    }

    async fn update_claim_status(
        &mut self,
        claim_id: &str,
        status: ClaimStatus,
    ) -> Result<(), AppError> {
        self.pending
            .status_updates
            .push((claim_id.to_string(), status));
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        let mut state = self.deps.state.lock().unwrap();
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        state.committed.reflections.extend(self.pending.reflections);
        for (claim_id, status) in self.pending.status_updates {
            let claim = state
                .committed
                .claims
                .iter_mut()
                .find(|claim| claim.claim_id == claim_id)
                .ok_or_else(|| AppError::Message(format!("missing claim: {claim_id}")))?;
            claim.status = status;
        }
        Ok(())
    }
}
