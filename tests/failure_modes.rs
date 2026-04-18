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
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery,
        IdGenerator, IdentityStore, IngestTransaction, IngestTransactionRunner,
        ReflectionTransaction, ReflectionTransactionRunner, StoredClaim, StoredEvent,
        StoredReflection,
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

    let result = execute_reflection(&deps, test_support::conflicting_reflection())
        .await
        .unwrap();

    let original_claim = deps
        .claim("claim-conflict")
        .expect("original conflicting claim should remain in history");
    assert_eq!(original_claim.status, ClaimStatus::Disputed);
    assert_ne!(original_claim.status, ClaimStatus::Superseded);
    assert_eq!(result.replacement_claim_id, None);
    assert!(
        deps.claim("id-1:replacement").is_none(),
        "conflict-only reflections must not create a replacement claim"
    );
    assert_eq!(
        deps.claim_count(),
        1,
        "marking a conflict should preserve the original claim instead of replacing it"
    );
}

#[tokio::test]
async fn reflection_rejects_identity_or_commitment_updates_without_resolved_evidence() {
    let deps = test_support::deps_for_failure_modes();

    let result =
        execute_reflection(&deps, test_support::reflection_updates_without_evidence()).await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert_eq!(
        deps.commitments(),
        vec![Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );
    assert!(deps.reflection("id-1").is_none());
}

#[tokio::test]
async fn reflection_rejects_empty_identity_update_even_with_supporting_evidence() {
    let deps = test_support::deps_for_failure_modes();

    let result = execute_reflection(
        &deps,
        test_support::reflection_updates_with_empty_identity(),
    )
    .await;

    assert!(matches!(result, Err(AppError::InvalidParams(_))));
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert!(deps.reflection("id-1").is_none());
}

#[tokio::test]
async fn reflection_commit_failure_rolls_back_claim_identity_commitment_and_audit_updates() {
    let deps = test_support::deps_with_fail_point(FailPoint::CommitReflection);

    let result = execute_reflection(&deps, test_support::reflection_updates_with_evidence()).await;

    assert!(
        matches!(result, Err(AppError::Message(message)) if message == "injected reflection commit failure")
    );
    assert_eq!(
        deps.claim("claim-conflict")
            .expect("original claim should remain")
            .status,
        ClaimStatus::Active
    );
    assert!(deps.claim("id-1:replacement").is_none());
    assert_eq!(
        deps.identity().canonical_claims(),
        &["identity:self=architect".to_string()]
    );
    assert_eq!(
        deps.commitments(),
        vec![Commitment::new(
            Owner::Self_,
            "forbid:write_identity_core_directly",
        )]
    );
    assert!(deps.reflection("id-1").is_none());
    assert!(deps.evidence_links().is_empty());
}

#[tokio::test]
async fn snapshot_budget_deduplicates_recent_duplicate_evidence_before_truncating() {
    let deps = test_support::deps_for_failure_modes();

    let snapshot = execute_build_snapshot(&deps, test_support::budgeted_snapshot())
        .await
        .unwrap()
        .snapshot;

    assert_eq!(
        snapshot.evidence,
        vec![
            "event:recent-duplicate".to_string(),
            "event:anchor".to_string(),
            "event:baseline".to_string(),
        ],
        "the current data model only tracks ordered event references, so this regression models a recent-event hijack as duplicate recent references exhausting the budget"
    );
}

mod test_support {
    use super::*;

    pub fn deps_for_failure_modes() -> FailureModeDeps {
        FailureModeDeps::new(State::default())
    }

    pub fn deps_with_fail_point(fail_point: FailPoint) -> FailureModeDeps {
        FailureModeDeps::new(State {
            fail_point: Some(fail_point),
            ..State::default()
        })
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
            Vec::new(),
        )
    }

    pub fn reflection_updates_without_evidence() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Identity-only updates still require supporting evidence."),
            "claim-conflict",
            None,
            Vec::new(),
        )
        .with_identity_update(vec!["identity:self=principal_architect".to_string()])
        .with_commitment_updates(vec![Commitment::new(
            Owner::Self_,
            "prefer:reflect_before_identity_changes",
        )])
    }

    pub fn reflection_updates_with_evidence() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Reflection should replace claim, identity, and commitments together."),
            "claim-conflict",
            Some(ClaimDraft::new(
                Owner::Self_,
                "self.role",
                "is",
                "principal_architect",
                Mode::Observed,
            )),
            vec!["evt-reflection-1".to_string()],
        )
        .with_identity_update(vec!["identity:self=principal_architect".to_string()])
        .with_commitment_updates(vec![Commitment::new(
            Owner::Self_,
            "prefer:reflect_before_identity_changes",
        )])
    }

    pub fn reflection_updates_with_empty_identity() -> ReflectionInput {
        ReflectionInput::new(
            Reflection::new("Identity updates cannot clear the canonical identity list."),
            "claim-conflict",
            None,
            vec!["evt-reflection-1".to_string()],
        )
        .with_identity_update(Vec::new())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailPoint {
    CommitReflection,
}

#[derive(Clone)]
struct State {
    committed: CommittedState,
    now: DateTime<Utc>,
    next_id: usize,
    fail_point: Option<FailPoint>,
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
    evidence_links: Vec<(String, String)>,
    reflections: Vec<StoredReflection>,
    status_updates: Vec<(String, ClaimStatus)>,
    identity: Option<IdentityCore>,
    commitments: Option<Vec<Commitment>>,
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
                    "event:recent-duplicate".to_string(),
                    "event:recent-duplicate".to_string(),
                    "event:recent-duplicate".to_string(),
                    "event:anchor".to_string(),
                    "event:baseline".to_string(),
                ],
                episode_references: vec!["episode:failure-modes".to_string()],
                reflections: Vec::new(),
                evidence_links: Vec::new(),
                events: vec![StoredEvent::new(
                    "evt-reflection-1".to_string(),
                    chrono::DateTime::parse_from_rfc3339("2026-03-23T10:01:00Z")
                        .unwrap()
                        .with_timezone(&Utc),
                    Event::new(Owner::World, EventKind::Observation, "evt-reflection-1"),
                )],
            },
            now: chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            next_id: 1,
            fail_point: None,
        }
    }
}

impl FailureModeDeps {
    fn new(state: State) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    fn claim(&self, claim_id: &str) -> Option<StoredClaim> {
        self.state
            .lock()
            .unwrap()
            .committed
            .claims
            .iter()
            .find(|claim| claim.claim_id == claim_id)
            .cloned()
    }

    fn claim_count(&self) -> usize {
        self.state.lock().unwrap().committed.claims.len()
    }

    fn identity(&self) -> IdentityCore {
        self.state.lock().unwrap().committed.identity.clone()
    }

    fn commitments(&self) -> Vec<Commitment> {
        self.state.lock().unwrap().committed.commitments.clone()
    }

    fn reflection(&self, reflection_id: &str) -> Option<StoredReflection> {
        self.state
            .lock()
            .unwrap()
            .committed
            .reflections
            .iter()
            .find(|reflection| reflection.reflection_id == reflection_id)
            .cloned()
    }

    fn evidence_links(&self) -> Vec<(String, String)> {
        self.state.lock().unwrap().committed.evidence_links.clone()
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

    async fn query_evidence_event_ids(
        &self,
        query: EvidenceQuery,
    ) -> Result<Vec<String>, AppError> {
        let mut events = self.state.lock().unwrap().committed.events.clone();

        if let Some(owner) = query.owner {
            events.retain(|event| event.event.owner() == owner);
        }

        if let Some(kind) = query.kind {
            events.retain(|event| event.event.kind() == kind);
        }

        events.sort_by(|lhs, rhs| {
            rhs.recorded_at
                .cmp(&lhs.recorded_at)
                .then_with(|| rhs.event_id.cmp(&lhs.event_id))
        });

        let limit = query.limit.unwrap_or(10);
        if limit == 0 {
            return Ok(Vec::new());
        }

        Ok(events
            .into_iter()
            .take(limit)
            .map(|event| event.event_id)
            .collect())
    }

    async fn has_event(&self, event_id: &str) -> Result<bool, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .events
            .iter()
            .any(|event| event.event_id == event_id))
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

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        self.pending.evidence_links.push((claim_id, event_id));
        Ok(())
    }

    async fn append_reflection(&mut self, reflection: StoredReflection) -> Result<(), AppError> {
        self.pending.reflections.push(reflection);
        Ok(())
    }

    async fn load_identity(&mut self) -> Result<IdentityCore, AppError> {
        if let Some(identity) = &self.pending.identity {
            return Ok(identity.clone());
        }

        Ok(self.deps.state.lock().unwrap().committed.identity.clone())
    }

    async fn replace_identity(&mut self, identity: IdentityCore) -> Result<(), AppError> {
        self.pending.identity = Some(identity);
        Ok(())
    }

    async fn load_commitments(&mut self) -> Result<Vec<Commitment>, AppError> {
        if let Some(commitments) = &self.pending.commitments {
            return Ok(commitments.clone());
        }

        Ok(self
            .deps
            .state
            .lock()
            .unwrap()
            .committed
            .commitments
            .clone())
    }

    async fn replace_commitments(&mut self, commitments: Vec<Commitment>) -> Result<(), AppError> {
        self.pending.commitments = Some(commitments);
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
        if self.deps.state.lock().unwrap().fail_point == Some(FailPoint::CommitReflection) {
            return Err(AppError::Message(
                "injected reflection commit failure".to_string(),
            ));
        }
        let mut state = self.deps.state.lock().unwrap();
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        state
            .committed
            .evidence_links
            .extend(self.pending.evidence_links);
        state.committed.reflections.extend(self.pending.reflections);
        if let Some(identity) = self.pending.identity {
            state.committed.identity = identity;
        }
        if let Some(commitments) = self.pending.commitments {
            state.committed.commitments = commitments;
        }
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
