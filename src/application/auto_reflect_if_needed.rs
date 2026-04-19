use chrono::{Duration, Utc};

use crate::{
    application::{
        build_self_snapshot::{self, BuildSelfSnapshotInput},
        run_reflection::{self, ReflectionInput},
    },
    domain::{
        commitment::Commitment,
        reflection::Reflection,
        self_revision::{SelfRevisionProposal, SelfRevisionRequest, TriggerType},
        types::{Namespace, Owner},
    },
    error::AppError,
    ports::{
        ClaimStore, Clock, CommitmentStore, EpisodeStore, EventStore, EvidenceQuery, IdGenerator,
        IdentityStore, ModelPort, ReflectionTransactionRunner, StoredTriggerLedgerEntry,
        TriggerLedgerStatus, TriggerLedgerStore,
    },
};

const FAILURE_TRIGGER_THRESHOLD: usize = 2;
const AUTO_REFLECTION_COOLDOWN_HOURS: i64 = 24;
const DEFAULT_SNAPSHOT_BUDGET: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum RecursionGuard {
    #[default]
    Allow,
    SkipAutoReflection,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AutoReflectInput {
    pub namespace: Namespace,
    pub trigger_type: TriggerType,
    #[serde(default)]
    pub trigger_hints: Vec<String>,
    #[serde(default)]
    pub recursion_guard: RecursionGuard,
}

impl AutoReflectInput {
    pub fn for_failure(namespace: Namespace, trigger_hints: Vec<String>) -> Self {
        Self::new(namespace, TriggerType::Failure, trigger_hints)
    }

    pub fn for_conflict(namespace: Namespace, trigger_hints: Vec<String>) -> Self {
        Self::new(namespace, TriggerType::Conflict, trigger_hints)
    }

    pub fn for_periodic(namespace: Namespace, trigger_hints: Vec<String>) -> Self {
        Self::new(namespace, TriggerType::Periodic, trigger_hints)
    }

    pub fn new(
        namespace: Namespace,
        trigger_type: TriggerType,
        trigger_hints: Vec<String>,
    ) -> Self {
        Self {
            namespace,
            trigger_type,
            trigger_hints,
            recursion_guard: RecursionGuard::Allow,
        }
    }

    pub fn with_recursion_guard(mut self, recursion_guard: RecursionGuard) -> Self {
        self.recursion_guard = recursion_guard;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AutoReflectResult {
    pub triggered: bool,
    pub trigger_type: Option<TriggerType>,
    pub reflection_id: Option<String>,
    pub ledger_status: Option<TriggerLedgerStatus>,
    pub reason: Option<String>,
}

impl AutoReflectResult {
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            triggered: false,
            trigger_type: None,
            reflection_id: None,
            ledger_status: None,
            reason: Some(reason.into()),
        }
    }

    pub fn not_triggered() -> Self {
        Self {
            triggered: false,
            trigger_type: None,
            reflection_id: None,
            ledger_status: None,
            reason: None,
        }
    }

    pub fn rejected(trigger_type: TriggerType, reason: impl Into<String>) -> Self {
        Self {
            triggered: false,
            trigger_type: Some(trigger_type),
            reflection_id: None,
            ledger_status: Some(TriggerLedgerStatus::Rejected),
            reason: Some(reason.into()),
        }
    }

    pub fn suppressed(trigger_type: TriggerType) -> Self {
        Self {
            triggered: false,
            trigger_type: Some(trigger_type),
            reflection_id: None,
            ledger_status: Some(TriggerLedgerStatus::Suppressed),
            reason: None,
        }
    }

    pub fn handled(trigger_type: TriggerType, reflection_id: String) -> Self {
        Self {
            triggered: true,
            trigger_type: Some(trigger_type),
            reflection_id: Some(reflection_id),
            ledger_status: Some(TriggerLedgerStatus::Handled),
            reason: None,
        }
    }
}

#[derive(Debug, Clone)]
struct TriggerCandidate {
    trigger_type: TriggerType,
    namespace: Namespace,
    trigger_hints: Vec<String>,
    trigger_key: String,
    evidence_event_ids: Vec<String>,
    should_consider: bool,
    episode_watermark: Option<u64>,
}

#[derive(Debug, Clone)]
struct ValidatedSelfRevision {
    identity_claims: Option<Vec<String>>,
    commitments: Option<Vec<Commitment>>,
}

#[derive(Debug, Clone)]
struct IdentityRevisionContext {
    proposed_values: Vec<String>,
    supporting_claim_count: usize,
    cross_episode_support_count: usize,
    has_high_conflict: bool,
    now: chrono::DateTime<Utc>,
    latest_handled_at: Option<chrono::DateTime<Utc>>,
    cooldown_until: Option<chrono::DateTime<Utc>>,
    patch_size: usize,
}

pub async fn execute<D>(deps: &D, input: AutoReflectInput) -> Result<AutoReflectResult, AppError>
where
    D: TriggerLedgerStore
        + EventStore
        + ClaimStore
        + CommitmentStore
        + IdentityStore
        + EpisodeStore
        + ReflectionTransactionRunner
        + ModelPort
        + Clock
        + IdGenerator
        + Sync,
{
    if input.recursion_guard == RecursionGuard::SkipAutoReflection {
        return Ok(AutoReflectResult::skipped("recursion guard enabled"));
    }

    let candidate = detect_trigger_candidate(deps, &input).await?;
    if !candidate.should_consider {
        return Ok(AutoReflectResult::not_triggered());
    }

    if trigger_is_suppressed(deps, &candidate).await? {
        record_suppressed_trigger(deps, &candidate).await?;
        return Ok(AutoReflectResult::suppressed(candidate.trigger_type));
    }

    let snapshot = build_revision_snapshot(deps, &candidate).await?;
    let proposal = deps
        .propose_self_revision(SelfRevisionRequest::new(
            candidate.trigger_type,
            candidate.namespace.clone(),
            snapshot,
            candidate.evidence_event_ids.clone(),
            candidate.trigger_hints.clone(),
        ))
        .await?;

    if !proposal.should_reflect {
        record_rejected_trigger(deps, &candidate, None).await?;
        return Ok(AutoReflectResult::rejected(
            candidate.trigger_type,
            proposal.rationale,
        ));
    }

    let validated = match validate_self_revision(deps, &candidate, &proposal).await {
        Ok(validated) => validated,
        Err(error) => {
            record_rejected_trigger(deps, &candidate, None).await?;
            return Err(error);
        }
    };

    match apply_validated_self_revision(deps, &candidate, &proposal, validated).await {
        Ok(result) => Ok(result),
        Err(error) => {
            record_rejected_trigger(deps, &candidate, None).await?;
            Err(error)
        }
    }
}

async fn detect_trigger_candidate<D>(
    deps: &D,
    input: &AutoReflectInput,
) -> Result<TriggerCandidate, AppError>
where
    D: EventStore + EpisodeStore + Sync,
{
    let evidence_event_ids = match input.trigger_type {
        TriggerType::Failure => {
            deps.query_evidence_event_ids(EvidenceQuery {
                owner: Some(Owner::Self_),
                kind: Some(crate::domain::types::EventKind::Action),
                limit: Some(5),
            })
            .await?
        }
        TriggerType::Conflict | TriggerType::Periodic => {
            deps.query_evidence_event_ids(EvidenceQuery {
                owner: None,
                kind: None,
                limit: Some(5),
            })
            .await?
        }
    };
    let episode_watermark =
        Some(dedupe_strings(deps.list_episode_references().await?).len() as u64);
    let should_consider = match input.trigger_type {
        TriggerType::Failure => {
            has_any_hint(&input.trigger_hints, &["failure", "rollback"])
                && evidence_event_ids.len() >= FAILURE_TRIGGER_THRESHOLD
        }
        TriggerType::Conflict => {
            has_any_hint(&input.trigger_hints, &["conflict", "rollback", "identity"])
                || !evidence_event_ids.is_empty()
        }
        TriggerType::Periodic => episode_watermark.unwrap_or(0) > 0,
    };

    Ok(TriggerCandidate {
        trigger_type: input.trigger_type,
        namespace: input.namespace.clone(),
        trigger_hints: input.trigger_hints.clone(),
        trigger_key: canonical_trigger_key(&input.namespace, input.trigger_type),
        evidence_event_ids,
        should_consider,
        episode_watermark,
    })
}

async fn trigger_is_suppressed<D>(deps: &D, candidate: &TriggerCandidate) -> Result<bool, AppError>
where
    D: TriggerLedgerStore + Clock + Sync,
{
    let Some(latest) = deps.latest_trigger_entry(&candidate.trigger_key).await? else {
        return Ok(false);
    };

    let now = deps.now().await?;
    if matches!(
        latest.status,
        TriggerLedgerStatus::Handled | TriggerLedgerStatus::Suppressed
    ) && latest
        .cooldown_until
        .is_some_and(|cooldown_until| cooldown_until > now)
    {
        return Ok(true);
    }

    if latest.status == TriggerLedgerStatus::Handled
        && !candidate.evidence_event_ids.is_empty()
        && latest.evidence_window == candidate.evidence_event_ids
    {
        return Ok(true);
    }

    if candidate.trigger_type == TriggerType::Periodic
        && latest.status == TriggerLedgerStatus::Handled
        && latest.episode_watermark.unwrap_or_default()
            >= candidate.episode_watermark.unwrap_or_default()
    {
        return Ok(true);
    }

    Ok(false)
}

async fn build_revision_snapshot<D>(
    deps: &D,
    candidate: &TriggerCandidate,
) -> Result<crate::domain::snapshot::SelfSnapshot, AppError>
where
    D: ClaimStore + CommitmentStore + IdentityStore + EventStore + EpisodeStore + Sync,
{
    Ok(build_self_snapshot::execute(
        deps,
        BuildSelfSnapshotInput::for_revision_window(
            candidate
                .evidence_event_ids
                .len()
                .max(DEFAULT_SNAPSHOT_BUDGET),
        ),
    )
    .await?
    .snapshot)
}

async fn validate_self_revision<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    proposal: &SelfRevisionProposal,
) -> Result<ValidatedSelfRevision, AppError>
where
    D: ClaimStore + EpisodeStore + TriggerLedgerStore + Clock + Sync,
{
    let identity_claims = if proposal.machine_patch.identity_patch.is_some() {
        Some(validate_identity_patch(
            proposal,
            &build_identity_revision_context(deps, candidate, proposal).await?,
        )?)
    } else {
        None
    };

    let commitments = if let Some(commitment_patch) = &proposal.machine_patch.commitment_patch {
        if candidate.evidence_event_ids.is_empty() {
            return Err(AppError::InvalidParams(
                "commitment auto-reflection updates require supporting evidence".to_string(),
            ));
        }

        if commitment_patch.commitments.is_empty() {
            return Err(AppError::InvalidParams(
                "commitment auto-reflection patch must include at least one commitment".to_string(),
            ));
        }

        Some(
            commitment_patch
                .commitments
                .iter()
                .cloned()
                .map(|commitment| Commitment::new(Owner::Self_, commitment))
                .collect(),
        )
    } else {
        None
    };

    if identity_claims.is_none() && commitments.is_none() {
        return Err(AppError::InvalidParams(
            "auto-reflection proposals must include at least one governed patch".to_string(),
        ));
    }

    Ok(ValidatedSelfRevision {
        identity_claims,
        commitments,
    })
}

async fn build_identity_revision_context<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    proposal: &SelfRevisionProposal,
) -> Result<IdentityRevisionContext, AppError>
where
    D: ClaimStore + EpisodeStore + TriggerLedgerStore + Clock + Sync,
{
    let active_claims = deps.list_active_claims().await?;
    let proposed_identity_claims = proposal
        .machine_patch
        .identity_patch
        .as_ref()
        .map(|patch| patch.canonical_claims.clone())
        .unwrap_or_default();
    let proposed_values = dedupe_strings(
        proposed_identity_claims
            .iter()
            .filter_map(|claim| claim.rsplit_once('=').map(|(_, value)| value.to_string()))
            .collect(),
    );
    let supporting_claims: Vec<_> = active_claims
        .iter()
        .filter(|claim| {
            claim.claim.namespace() == &candidate.namespace
                && proposed_values
                    .iter()
                    .any(|value| value == claim.claim.object())
        })
        .cloned()
        .collect();
    let supporting_claim_count = supporting_claims.len();
    let supporting_shapes = supporting_claims
        .iter()
        .fold(Vec::new(), |mut shapes, claim| {
            let shape = (
                claim.claim.subject().to_string(),
                claim.claim.predicate().to_string(),
            );
            if !shapes.contains(&shape) {
                shapes.push(shape);
            }
            shapes
        });
    let has_high_conflict = active_claims.iter().any(|claim| {
        claim.claim.namespace() == &candidate.namespace
            && supporting_shapes.iter().any(|(subject, predicate)| {
                claim.claim.subject() == subject
                    && claim.claim.predicate() == predicate
                    && !proposed_values
                        .iter()
                        .any(|value| value == claim.claim.object())
            })
    });
    let latest_entry = deps.latest_trigger_entry(&candidate.trigger_key).await?;
    let cross_episode_support_count = dedupe_strings(deps.list_episode_references().await?)
        .len()
        .min(supporting_claim_count);
    let now = deps.now().await?;

    Ok(IdentityRevisionContext {
        proposed_values,
        supporting_claim_count,
        cross_episode_support_count,
        has_high_conflict,
        now,
        latest_handled_at: latest_entry.as_ref().and_then(|entry| {
            (entry.status == TriggerLedgerStatus::Handled)
                .then_some(entry.handled_at)
                .flatten()
        }),
        cooldown_until: latest_entry.as_ref().and_then(|entry| {
            matches!(
                entry.status,
                TriggerLedgerStatus::Handled | TriggerLedgerStatus::Suppressed
            )
            .then_some(entry.cooldown_until)
            .flatten()
        }),
        patch_size: proposed_identity_claims.len(),
    })
}

fn validate_identity_patch(
    proposal: &SelfRevisionProposal,
    context: &IdentityRevisionContext,
) -> Result<Vec<String>, AppError> {
    ensure_min_supporting_claims(context, 3)?;
    ensure_cross_episode_support(context, 2)?;
    ensure_no_high_conflict(context)?;
    ensure_identity_cooldown_elapsed(context)?;
    ensure_identity_patch_limit(context, 2)?;
    Ok(materialize_identity_claims(proposal))
}

fn ensure_min_supporting_claims(
    context: &IdentityRevisionContext,
    minimum: usize,
) -> Result<(), AppError> {
    if context.supporting_claim_count < minimum {
        return Err(AppError::InvalidParams(format!(
            "identity auto-reflection requires at least {minimum} supporting claims"
        )));
    }

    Ok(())
}

fn ensure_cross_episode_support(
    context: &IdentityRevisionContext,
    minimum: usize,
) -> Result<(), AppError> {
    if context.cross_episode_support_count < minimum {
        return Err(AppError::InvalidParams(format!(
            "identity auto-reflection requires support across at least {minimum} episodes"
        )));
    }

    Ok(())
}

fn ensure_no_high_conflict(context: &IdentityRevisionContext) -> Result<(), AppError> {
    if context.has_high_conflict {
        return Err(AppError::InvalidParams(format!(
            "identity auto-reflection cannot proceed while high-conflict evidence remains active for {:?}",
            context.proposed_values
        )));
    }

    Ok(())
}

fn ensure_identity_cooldown_elapsed(context: &IdentityRevisionContext) -> Result<(), AppError> {
    if context
        .cooldown_until
        .is_some_and(|cooldown_until| cooldown_until > context.now)
    {
        return Err(AppError::InvalidParams(
            "identity auto-reflection cooldown has not elapsed".to_string(),
        ));
    }

    if context.latest_handled_at.is_some_and(|handled_at| {
        handled_at + Duration::hours(AUTO_REFLECTION_COOLDOWN_HOURS) > context.now
    }) {
        return Err(AppError::InvalidParams(
            "identity auto-reflection handled too recently".to_string(),
        ));
    }

    Ok(())
}

fn ensure_identity_patch_limit(
    context: &IdentityRevisionContext,
    maximum: usize,
) -> Result<(), AppError> {
    if context.patch_size == 0 || context.patch_size > maximum {
        return Err(AppError::InvalidParams(format!(
            "identity auto-reflection patch must contain between 1 and {maximum} claims"
        )));
    }

    Ok(())
}

fn materialize_identity_claims(proposal: &SelfRevisionProposal) -> Vec<String> {
    proposal
        .machine_patch
        .identity_patch
        .as_ref()
        .map(|patch| patch.canonical_claims.clone())
        .unwrap_or_default()
}

async fn apply_validated_self_revision<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    proposal: &SelfRevisionProposal,
    validated: ValidatedSelfRevision,
) -> Result<AutoReflectResult, AppError>
where
    D: EventStore + ReflectionTransactionRunner + Clock + IdGenerator + Sync,
{
    let handled_trigger_ledger_entry =
        build_trigger_entry(deps, candidate, TriggerLedgerStatus::Handled, None, None).await?;
    let reflection_input = ReflectionInput::record_only(
        Reflection::new(proposal.rationale.clone()),
        candidate.evidence_event_ids.clone(),
    )
    .with_optional_replacement_evidence_query(None)
    .with_handled_trigger_ledger_entry(handled_trigger_ledger_entry);
    let reflection_input = if let Some(identity_claims) = validated.identity_claims {
        reflection_input.with_identity_update(identity_claims)
    } else {
        reflection_input
    };
    let reflection_input = if let Some(commitments) = validated.commitments {
        reflection_input.with_commitment_updates(commitments)
    } else {
        reflection_input
    };
    let reflection = run_reflection::execute(deps, reflection_input).await?;

    Ok(AutoReflectResult::handled(
        candidate.trigger_type,
        reflection.reflection_id,
    ))
}

async fn record_suppressed_trigger<D>(
    deps: &D,
    candidate: &TriggerCandidate,
) -> Result<(), AppError>
where
    D: TriggerLedgerStore + Clock + IdGenerator + Sync,
{
    let preserved_cooldown_until = latest_live_cooldown_until(deps, &candidate.trigger_key).await?;
    record_trigger_entry(
        deps,
        candidate,
        TriggerLedgerStatus::Suppressed,
        None,
        preserved_cooldown_until,
    )
    .await
}

async fn record_rejected_trigger<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    reflection_id: Option<String>,
) -> Result<(), AppError>
where
    D: TriggerLedgerStore + Clock + IdGenerator + Sync,
{
    record_trigger_entry(
        deps,
        candidate,
        TriggerLedgerStatus::Rejected,
        reflection_id,
        None,
    )
    .await
}

async fn build_trigger_entry<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    status: TriggerLedgerStatus,
    reflection_id: Option<String>,
    cooldown_until_override: Option<chrono::DateTime<Utc>>,
) -> Result<StoredTriggerLedgerEntry, AppError>
where
    D: Clock + IdGenerator + Sync,
{
    let now = deps.now().await?;
    let handled_at = (status == TriggerLedgerStatus::Handled).then_some(now);
    let cooldown_until = cooldown_until_override.or(match status {
        TriggerLedgerStatus::Handled | TriggerLedgerStatus::Suppressed => {
            Some(now + Duration::hours(AUTO_REFLECTION_COOLDOWN_HOURS))
        }
        TriggerLedgerStatus::Pending | TriggerLedgerStatus::Rejected => None,
    });

    Ok(StoredTriggerLedgerEntry {
        ledger_id: deps.next_id().await?,
        trigger_type: candidate.trigger_type,
        namespace: candidate.namespace.clone(),
        trigger_key: candidate.trigger_key.clone(),
        status,
        evidence_window: candidate.evidence_event_ids.clone(),
        handled_at,
        cooldown_until,
        episode_watermark: candidate.episode_watermark,
        reflection_id,
    })
}

async fn record_trigger_entry<D>(
    deps: &D,
    candidate: &TriggerCandidate,
    status: TriggerLedgerStatus,
    reflection_id: Option<String>,
    cooldown_until_override: Option<chrono::DateTime<Utc>>,
) -> Result<(), AppError>
where
    D: TriggerLedgerStore + Clock + IdGenerator + Sync,
{
    deps.record_trigger_attempt(
        build_trigger_entry(
            deps,
            candidate,
            status,
            reflection_id,
            cooldown_until_override,
        )
        .await?,
    )
    .await
}

async fn latest_live_cooldown_until<D>(
    deps: &D,
    trigger_key: &str,
) -> Result<Option<chrono::DateTime<Utc>>, AppError>
where
    D: TriggerLedgerStore + Clock + Sync,
{
    let Some(latest) = deps.latest_trigger_entry(trigger_key).await? else {
        return Ok(None);
    };
    let now = deps.now().await?;

    Ok(matches!(
        latest.status,
        TriggerLedgerStatus::Handled | TriggerLedgerStatus::Suppressed
    )
    .then_some(latest.cooldown_until)
    .flatten()
    .filter(|cooldown_until| *cooldown_until > now))
}

fn canonical_trigger_key(namespace: &Namespace, trigger_type: TriggerType) -> String {
    format!(
        "{}:{}",
        namespace.as_str(),
        trigger_type_label(trigger_type)
    )
}

fn trigger_type_label(trigger_type: TriggerType) -> &'static str {
    match trigger_type {
        TriggerType::Conflict => "conflict",
        TriggerType::Failure => "failure",
        TriggerType::Periodic => "periodic",
    }
}

fn has_any_hint(hints: &[String], expected: &[&str]) -> bool {
    hints.iter().any(|hint| {
        expected
            .iter()
            .any(|expected_hint| hint.eq_ignore_ascii_case(expected_hint))
    })
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
