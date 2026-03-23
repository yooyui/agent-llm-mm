use std::sync::{Arc, Mutex};

use agent_llm_mm::{
    application::{
        build_self_snapshot::{BuildSelfSnapshotInput, execute as execute_build_snapshot},
        decide_with_snapshot::{DecideWithSnapshotInput, execute as execute_decide},
        ingest_interaction::{IngestInput, execute as execute_ingest},
        run_reflection::{ReflectionInput, execute as execute_reflection},
    },
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        identity_core::IdentityCore,
        reflection::Reflection,
        snapshot::{SelfSnapshot, SnapshotBudget},
        types::{EventKind, Mode, Owner},
    },
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, CommitmentStore, EventStore, IdGenerator, IdentityStore,
        IngestTransaction, IngestTransactionRunner, ModelDecision, ModelInput, ModelPort,
        ReflectionStore, ReflectionTransaction, ReflectionTransactionRunner, StoredClaim,
        StoredEvent, StoredReflection,
    },
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};

#[tokio::test]
async fn ingest_writes_events_before_claims_and_commits_payloads() {
    let deps = test_support::in_memory_deps();

    let result = execute_ingest(&deps, test_support::ingest_input())
        .await
        .unwrap();

    assert_eq!(result.event_id, "id-1");
    assert_eq!(
        deps.log(),
        vec![
            "append_event",
            "record_event_in_episode",
            "upsert_claim",
            "link_evidence"
        ]
    );

    let event = deps.event("id-1").expect("event should be committed");
    assert_eq!(event.event.summary(), "The user asked for stronger memory.");
    assert_eq!(event.event.kind(), EventKind::Conversation);

    let claim = deps
        .claim("id-1:claim:0")
        .expect("derived claim should be committed");
    assert_eq!(claim.claim.subject(), "self.role");
    assert_eq!(claim.claim.object(), "architect");
    assert_eq!(claim.status, ClaimStatus::Active);

    assert_eq!(
        deps.evidence_links(),
        vec![("id-1:claim:0".to_string(), "id-1".to_string())]
    );
    assert_eq!(
        deps.episode_links(),
        vec![("episode:task-4".to_string(), "id-1".to_string())]
    );
}

#[tokio::test]
async fn reflection_supersedes_old_claim_and_persists_audit_record() {
    let deps = test_support::in_memory_deps();

    let result = execute_reflection(&deps, test_support::reflection_input())
        .await
        .unwrap();

    assert_eq!(result.reflection_id, "id-1");
    assert_eq!(
        result.replacement_claim_id.as_deref(),
        Some("id-1:replacement")
    );

    let old_claim = deps.claim("claim-old").expect("old claim should remain");
    assert_eq!(old_claim.status, ClaimStatus::Superseded);

    let new_claim = deps
        .claim("id-1:replacement")
        .expect("replacement claim should be committed");
    assert_eq!(new_claim.claim.object(), "senior_architect");
    assert_eq!(new_claim.status, ClaimStatus::Active);

    let reflection = deps
        .reflection("id-1")
        .expect("reflection audit record should be committed");
    assert_eq!(reflection.superseded_claim_id.as_deref(), Some("claim-old"));
    assert_eq!(
        reflection.replacement_claim_id.as_deref(),
        Some("id-1:replacement")
    );
}

#[tokio::test]
async fn build_self_snapshot_returns_store_backed_snapshot_and_respects_budget() {
    let deps = test_support::snapshot_deps();

    let result = execute_build_snapshot(
        &deps,
        BuildSelfSnapshotInput {
            budget: SnapshotBudget::new(2),
        },
    )
    .await
    .unwrap();

    assert_eq!(
        result.snapshot.identity,
        vec!["identity:self=architect".to_string()]
    );
    assert_eq!(
        result.snapshot.commitments,
        vec!["forbid:write_identity_core_directly".to_string()]
    );
    assert_eq!(
        result.snapshot.claims,
        vec!["self.role is architect".to_string()]
    );
    assert_eq!(
        result.snapshot.evidence,
        vec!["event:evt-1".to_string(), "event:evt-2".to_string()]
    );
    assert_eq!(
        result.snapshot.episodes,
        vec!["episode:task-4".to_string(), "episode:memory".to_string()]
    );
}

#[tokio::test]
async fn decide_with_snapshot_does_not_call_model_when_gate_blocks() {
    let deps = test_support::in_memory_deps();

    let result = execute_decide(
        &deps,
        DecideWithSnapshotInput {
            task: "decide on identity write".to_string(),
            action: "write_identity_core_directly".to_string(),
            snapshot: test_support::decision_snapshot(),
        },
    )
    .await
    .unwrap();

    assert!(result.blocked);
    assert!(result.decision.is_none());
    assert_eq!(deps.model_call_count(), 0);
}

#[tokio::test]
async fn decide_with_snapshot_calls_model_when_gate_allows() {
    let deps = test_support::in_memory_deps();

    let result = execute_decide(
        &deps,
        DecideWithSnapshotInput {
            task: "read identity safely".to_string(),
            action: "read_identity_core".to_string(),
            snapshot: test_support::decision_snapshot(),
        },
    )
    .await
    .unwrap();

    assert!(!result.blocked);
    assert_eq!(
        result.decision,
        Some(ModelDecision::new("Proceed".to_string()))
    );
    assert_eq!(deps.model_call_count(), 1);

    let model_input = deps.last_model_input().expect("model should be called");
    assert_eq!(model_input.task, "read identity safely");
    assert_eq!(model_input.action, "read_identity_core");
    assert_eq!(
        model_input.snapshot.identity,
        vec!["identity:self=architect".to_string()]
    );
}

#[tokio::test]
async fn ingest_failure_before_commit_does_not_leak_partial_writes() {
    let deps = test_support::deps_with_fail_point(FailPoint::LinkEvidence);

    let result = execute_ingest(&deps, test_support::ingest_input()).await;

    assert!(result.is_err());
    assert!(
        deps.events().is_empty(),
        "event must not be visible before commit"
    );
    assert!(
        deps.claim("id-1:claim:0").is_none(),
        "claim must not be visible before commit"
    );
    assert!(
        deps.evidence_links().is_empty(),
        "link must not leak before commit"
    );
    assert!(
        deps.episode_links().is_empty(),
        "episode link must not leak before commit"
    );
    assert!(
        deps.log().is_empty(),
        "staged operations must not leak into the log"
    );
}

mod test_support {
    use super::*;

    pub fn in_memory_deps() -> InMemoryDeps {
        InMemoryDeps::new(State::default())
    }

    pub fn snapshot_deps() -> InMemoryDeps {
        let mut state = State::default();
        state.committed.events = vec![
            StoredEvent::new(
                "evt-1".to_string(),
                fixed_now(),
                Event::new(Owner::User, EventKind::Observation, "evt-1"),
            ),
            StoredEvent::new(
                "evt-2".to_string(),
                fixed_now(),
                Event::new(Owner::User, EventKind::Observation, "evt-2"),
            ),
            StoredEvent::new(
                "evt-3".to_string(),
                fixed_now(),
                Event::new(Owner::User, EventKind::Observation, "evt-3"),
            ),
        ];
        state.committed.episodes = vec![
            ("episode:task-4".to_string(), "evt-1".to_string()),
            ("episode:memory".to_string(), "evt-2".to_string()),
        ];
        state.committed.claims = vec![
            StoredClaim::new(
                "claim-active".to_string(),
                ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                ClaimStatus::Active,
            ),
            StoredClaim::new(
                "claim-superseded".to_string(),
                ClaimDraft::new(
                    Owner::Self_,
                    "self.role",
                    "is",
                    "old_architect",
                    Mode::Observed,
                ),
                ClaimStatus::Superseded,
            ),
        ];
        InMemoryDeps::new(state)
    }

    pub fn deps_with_fail_point(fail_point: FailPoint) -> InMemoryDeps {
        let mut state = State::default();
        state.fail_point = Some(fail_point);
        InMemoryDeps::new(state)
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

    pub fn decision_snapshot() -> SelfSnapshot {
        SelfSnapshot {
            identity: vec!["identity:self=architect".to_string()],
            commitments: vec!["forbid:write_identity_core_directly".to_string()],
            claims: vec!["self.role is architect".to_string()],
            evidence: vec!["event:evt-1".to_string()],
            episodes: vec!["episode:task-4".to_string()],
        }
    }

    fn fixed_now() -> DateTime<Utc> {
        chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailPoint {
    LinkEvidence,
}

#[derive(Clone)]
struct InMemoryDeps {
    state: Arc<Mutex<State>>,
}

#[derive(Clone)]
struct State {
    committed: CommittedState,
    next_id: usize,
    now: DateTime<Utc>,
    fail_point: Option<FailPoint>,
    model_calls: Vec<ModelInput>,
}

#[derive(Debug, Clone, Default)]
struct CommittedState {
    log: Vec<String>,
    events: Vec<StoredEvent>,
    claims: Vec<StoredClaim>,
    evidence_links: Vec<(String, String)>,
    episodes: Vec<(String, String)>,
    reflections: Vec<StoredReflection>,
    commitments: Vec<Commitment>,
    identity: Option<IdentityCore>,
}

#[derive(Debug, Default)]
struct PendingIngest {
    log: Vec<String>,
    events: Vec<StoredEvent>,
    claims: Vec<StoredClaim>,
    evidence_links: Vec<(String, String)>,
    episodes: Vec<(String, String)>,
}

#[derive(Debug, Default)]
struct PendingReflection {
    claims: Vec<StoredClaim>,
    reflections: Vec<StoredReflection>,
    status_updates: Vec<(String, ClaimStatus)>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            committed: CommittedState {
                commitments: vec![Commitment::new(
                    Owner::Self_,
                    "forbid:write_identity_core_directly",
                )],
                identity: Some(IdentityCore::new(vec![
                    "identity:self=architect".to_string(),
                ])),
                claims: vec![StoredClaim::new(
                    "claim-old".to_string(),
                    ClaimDraft::new(Owner::Self_, "self.role", "is", "architect", Mode::Observed),
                    ClaimStatus::Active,
                )],
                ..CommittedState::default()
            },
            next_id: 1,
            now: chrono::DateTime::parse_from_rfc3339("2026-03-23T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            fail_point: None,
            model_calls: Vec::new(),
        }
    }
}

impl InMemoryDeps {
    fn new(state: State) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    fn log(&self) -> Vec<String> {
        self.state.lock().unwrap().committed.log.clone()
    }

    fn events(&self) -> Vec<StoredEvent> {
        self.state.lock().unwrap().committed.events.clone()
    }

    fn event(&self, event_id: &str) -> Option<StoredEvent> {
        self.events()
            .into_iter()
            .find(|event| event.event_id == event_id)
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

    fn evidence_links(&self) -> Vec<(String, String)> {
        self.state.lock().unwrap().committed.evidence_links.clone()
    }

    fn episode_links(&self) -> Vec<(String, String)> {
        self.state.lock().unwrap().committed.episodes.clone()
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

    fn model_call_count(&self) -> usize {
        self.state.lock().unwrap().model_calls.len()
    }

    fn last_model_input(&self) -> Option<ModelInput> {
        self.state.lock().unwrap().model_calls.last().cloned()
    }
}

fn upsert_claim(target: &mut Vec<StoredClaim>, next_claim: StoredClaim) {
    if let Some(existing) = target
        .iter_mut()
        .find(|claim| claim.claim_id == next_claim.claim_id)
    {
        *existing = next_claim;
    } else {
        target.push(next_claim);
    }
}

#[async_trait]
impl EventStore for InMemoryDeps {
    async fn append_event(&self, event: StoredEvent) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        state.committed.log.push("append_event".to_string());
        state.committed.events.push(event);
        Ok(())
    }

    async fn list_event_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .events
            .iter()
            .map(StoredEvent::event_reference)
            .collect())
    }
}

#[async_trait]
impl ClaimStore for InMemoryDeps {
    async fn upsert_claim(&self, claim: StoredClaim) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        state.committed.log.push("upsert_claim".to_string());
        upsert_claim(&mut state.committed.claims, claim);
        Ok(())
    }

    async fn link_evidence(&self, claim_id: String, event_id: String) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        state.committed.log.push("link_evidence".to_string());
        state.committed.evidence_links.push((claim_id, event_id));
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
        if let Some(claim) = state
            .committed
            .claims
            .iter_mut()
            .find(|claim| claim.claim_id == claim_id)
        {
            claim.status = status;
        }
        Ok(())
    }
}

#[async_trait]
impl IdentityStore for InMemoryDeps {
    async fn load_identity(&self) -> Result<IdentityCore, AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .identity
            .clone()
            .ok_or_else(|| AppError::Message("missing identity".to_string()))
    }

    async fn save_identity(&self, identity: IdentityCore) -> Result<(), AppError> {
        self.state.lock().unwrap().committed.identity = Some(identity);
        Ok(())
    }
}

#[async_trait]
impl CommitmentStore for InMemoryDeps {
    async fn list_commitments(&self) -> Result<Vec<Commitment>, AppError> {
        Ok(self.state.lock().unwrap().committed.commitments.clone())
    }
}

#[async_trait]
impl agent_llm_mm::ports::EpisodeStore for InMemoryDeps {
    async fn record_event_in_episode(
        &self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError> {
        let mut state = self.state.lock().unwrap();
        state
            .committed
            .log
            .push("record_event_in_episode".to_string());
        state.committed.episodes.push((episode_reference, event_id));
        Ok(())
    }

    async fn list_episode_references(&self) -> Result<Vec<String>, AppError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .committed
            .episodes
            .iter()
            .map(|(episode_reference, _)| episode_reference.clone())
            .collect())
    }
}

#[async_trait]
impl ReflectionStore for InMemoryDeps {
    async fn append_reflection(&self, reflection: StoredReflection) -> Result<(), AppError> {
        self.state
            .lock()
            .unwrap()
            .committed
            .reflections
            .push(reflection);
        Ok(())
    }
}

#[async_trait]
impl ModelPort for InMemoryDeps {
    async fn decide(&self, input: ModelInput) -> Result<ModelDecision, AppError> {
        self.state.lock().unwrap().model_calls.push(input);
        Ok(ModelDecision::new("Proceed".to_string()))
    }
}

#[async_trait]
impl Clock for InMemoryDeps {
    async fn now(&self) -> Result<DateTime<Utc>, AppError> {
        Ok(self.state.lock().unwrap().now)
    }
}

#[async_trait]
impl IdGenerator for InMemoryDeps {
    async fn next_id(&self) -> Result<String, AppError> {
        let mut state = self.state.lock().unwrap();
        let next_id = format!("id-{}", state.next_id);
        state.next_id += 1;
        Ok(next_id)
    }
}

#[async_trait]
impl IngestTransactionRunner for InMemoryDeps {
    async fn begin_ingest_transaction(
        &self,
    ) -> Result<Box<dyn IngestTransaction + Send + '_>, AppError> {
        Ok(Box::new(InMemoryIngestTransaction {
            deps: self.clone(),
            pending: PendingIngest::default(),
        }))
    }
}

#[async_trait]
impl ReflectionTransactionRunner for InMemoryDeps {
    async fn begin_reflection_transaction(
        &self,
    ) -> Result<Box<dyn ReflectionTransaction + Send + '_>, AppError> {
        Ok(Box::new(InMemoryReflectionTransaction {
            deps: self.clone(),
            pending: PendingReflection::default(),
        }))
    }
}

struct InMemoryIngestTransaction {
    deps: InMemoryDeps,
    pending: PendingIngest,
}

#[async_trait]
impl IngestTransaction for InMemoryIngestTransaction {
    async fn append_event(&mut self, event: StoredEvent) -> Result<(), AppError> {
        self.pending.log.push("append_event".to_string());
        self.pending.events.push(event);
        Ok(())
    }

    async fn record_event_in_episode(
        &mut self,
        episode_reference: String,
        event_id: String,
    ) -> Result<(), AppError> {
        self.pending.log.push("record_event_in_episode".to_string());
        self.pending.episodes.push((episode_reference, event_id));
        Ok(())
    }

    async fn upsert_claim(&mut self, claim: StoredClaim) -> Result<(), AppError> {
        self.pending.log.push("upsert_claim".to_string());
        self.pending.claims.push(claim);
        Ok(())
    }

    async fn link_evidence(&mut self, claim_id: String, event_id: String) -> Result<(), AppError> {
        let should_fail = self.state_fail_point() == Some(FailPoint::LinkEvidence);
        if should_fail {
            return Err(AppError::Message("injected link failure".to_string()));
        }

        self.pending.log.push("link_evidence".to_string());
        self.pending.evidence_links.push((claim_id, event_id));
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        let mut state = self.deps.state.lock().unwrap();
        state.committed.log.extend(self.pending.log);
        state.committed.events.extend(self.pending.events);
        for claim in self.pending.claims {
            upsert_claim(&mut state.committed.claims, claim);
        }
        state
            .committed
            .evidence_links
            .extend(self.pending.evidence_links);
        state.committed.episodes.extend(self.pending.episodes);
        Ok(())
    }
}

impl InMemoryIngestTransaction {
    fn state_fail_point(&self) -> Option<FailPoint> {
        self.deps.state.lock().unwrap().fail_point
    }
}

struct InMemoryReflectionTransaction {
    deps: InMemoryDeps,
    pending: PendingReflection,
}

#[async_trait]
impl ReflectionTransaction for InMemoryReflectionTransaction {
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
            if let Some(claim) = state
                .committed
                .claims
                .iter_mut()
                .find(|claim| claim.claim_id == claim_id)
            {
                claim.status = status;
            }
        }
        Ok(())
    }
}
