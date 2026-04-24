use std::collections::BTreeSet;

use crate::ports::EvidenceQuery;
use crate::{
    application::{
        auto_reflect_if_needed::AutoReflectInput, build_self_snapshot::BuildSelfSnapshotInput,
        decide_with_snapshot::DecideWithSnapshotInput, ingest_interaction::IngestInput,
        run_reflection::ReflectionInput,
    },
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        event::Event,
        reflection::{Reflection, ReflectionIdentityUpdate},
        self_revision::TriggerType,
        snapshot::{SelfSnapshot, SnapshotBudget},
        types::{EventKind, Mode, Namespace, Owner},
    },
    error::AppError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum OwnerDto {
    Self_,
    User,
    World,
    Unknown,
}

impl From<OwnerDto> for Owner {
    fn from(value: OwnerDto) -> Self {
        match value {
            OwnerDto::Self_ => Owner::Self_,
            OwnerDto::User => Owner::User,
            OwnerDto::World => Owner::World,
            OwnerDto::Unknown => Owner::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ModeDto {
    Observed,
    Said,
    Acted,
    Inferred,
    Draft,
}

impl From<ModeDto> for Mode {
    fn from(value: ModeDto) -> Self {
        match value {
            ModeDto::Observed => Mode::Observed,
            ModeDto::Said => Mode::Said,
            ModeDto::Acted => Mode::Acted,
            ModeDto::Inferred => Mode::Inferred,
            ModeDto::Draft => Mode::Draft,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum EventKindDto {
    Observation,
    Conversation,
    Action,
    Reflection,
}

impl From<EventKindDto> for EventKind {
    fn from(value: EventKindDto) -> Self {
        match value {
            EventKindDto::Observation => EventKind::Observation,
            EventKindDto::Conversation => EventKind::Conversation,
            EventKindDto::Action => EventKind::Action,
            EventKindDto::Reflection => EventKind::Reflection,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EventDto {
    pub owner: OwnerDto,
    pub kind: EventKindDto,
    pub summary: String,
}

impl From<EventDto> for Event {
    fn from(value: EventDto) -> Self {
        Event::new(value.owner.into(), value.kind.into(), value.summary)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClaimDraftDto {
    pub owner: OwnerDto,
    pub namespace: Option<String>,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub mode: ModeDto,
}

impl TryFrom<ClaimDraftDto> for ClaimDraft {
    type Error = crate::domain::DomainError;

    fn try_from(value: ClaimDraftDto) -> Result<Self, Self::Error> {
        let owner = Owner::from(value.owner);
        let mode = Mode::from(value.mode);
        let namespace = value.namespace.map(Namespace::parse).transpose()?;

        Ok(match namespace {
            Some(namespace) => ClaimDraft::new_with_namespace(
                owner,
                namespace,
                value.subject,
                value.predicate,
                value.object,
                mode,
            ),
            None => ClaimDraft::new(owner, value.subject, value.predicate, value.object, mode),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CommitmentDto {
    pub owner: OwnerDto,
    pub description: String,
}

impl From<CommitmentDto> for Commitment {
    fn from(value: CommitmentDto) -> Self {
        Commitment::new(value.owner.into(), value.description)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IngestInteractionParams {
    pub event: EventDto,
    pub claim_drafts: Vec<ClaimDraftDto>,
    pub episode_reference: Option<String>,
    #[serde(default)]
    pub trigger_hints: Vec<String>,
}

impl TryFrom<IngestInteractionParams> for IngestInput {
    type Error = crate::domain::DomainError;

    fn try_from(value: IngestInteractionParams) -> Result<Self, Self::Error> {
        Ok(IngestInput::new(
            value.event.into(),
            value
                .claim_drafts
                .into_iter()
                .map(ClaimDraft::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            value.episode_reference,
        ))
    }
}

impl IngestInteractionParams {
    fn auto_reflect_namespace(&self) -> Result<Namespace, AppError> {
        if self.claim_drafts.is_empty() {
            return Ok(Namespace::for_owner(self.event.owner.into()));
        }

        let mut namespaces = BTreeSet::new();
        for draft in &self.claim_drafts {
            let namespace = match draft.namespace.as_deref() {
                Some(namespace) => {
                    Namespace::parse(namespace.to_string()).map_err(AppError::from)?
                }
                None => Namespace::for_owner(Owner::from(draft.owner)),
            };
            namespaces.insert(namespace.as_str().to_string());
        }

        match namespaces.len() {
            1 => Namespace::parse(
                namespaces
                    .into_iter()
                    .next()
                    .expect("single namespace must exist"),
            )
            .map_err(AppError::from),
            _ => Err(AppError::InvalidParams(format!(
                "ambiguous auto-reflection namespace derived from claim drafts: {}",
                namespaces.into_iter().collect::<Vec<_>>().join(", ")
            ))),
        }
    }
}

impl AutoReflectInput {
    pub fn from_ingest(params: &IngestInteractionParams) -> Result<Self, AppError> {
        Ok(Self::new(
            params.auto_reflect_namespace()?,
            ingest_trigger_type_from_hints(&params.trigger_hints),
            params.trigger_hints.clone(),
        ))
    }
}

// Ingest only upgrades to the conflict hook for explicit conflict/identity hints.
// Rollback-only hints stay on the existing failure path.
fn ingest_trigger_type_from_hints(trigger_hints: &[String]) -> TriggerType {
    if trigger_hints
        .iter()
        .any(|hint| matches!(hint.to_ascii_lowercase().as_str(), "conflict" | "identity"))
    {
        TriggerType::Conflict
    } else {
        TriggerType::Failure
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BuildSelfSnapshotParams {
    pub budget: usize,
    #[serde(default)]
    pub auto_reflect_namespace: Option<String>,
}

impl From<BuildSelfSnapshotParams> for BuildSelfSnapshotInput {
    fn from(value: BuildSelfSnapshotParams) -> Self {
        BuildSelfSnapshotInput {
            budget: SnapshotBudget::new(value.budget),
        }
    }
}

impl BuildSelfSnapshotParams {
    fn auto_reflect_namespace(&self) -> Result<Option<Namespace>, AppError> {
        self.auto_reflect_namespace
            .as_deref()
            .map(|namespace| Namespace::parse(namespace.to_string()).map_err(AppError::from))
            .transpose()
    }
}

impl AutoReflectInput {
    pub fn from_build_snapshot(params: &BuildSelfSnapshotParams) -> Result<Option<Self>, AppError> {
        params
            .auto_reflect_namespace()?
            .map(|namespace| Ok(Self::for_periodic(namespace, vec!["periodic".to_string()])))
            .transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SelfSnapshotDto {
    pub identity: Vec<String>,
    pub commitments: Vec<String>,
    pub claims: Vec<String>,
    pub evidence: Vec<String>,
    pub episodes: Vec<String>,
}

impl From<SelfSnapshotDto> for SelfSnapshot {
    fn from(value: SelfSnapshotDto) -> Self {
        SelfSnapshot {
            identity: value.identity,
            commitments: value.commitments,
            claims: value.claims,
            evidence: value.evidence,
            episodes: value.episodes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DecideWithSnapshotParams {
    pub task: String,
    pub action: String,
    pub snapshot: SelfSnapshotDto,
    #[serde(default)]
    pub trigger_hints: Vec<String>,
    #[serde(default)]
    pub auto_reflect_namespace: Option<String>,
}

impl From<DecideWithSnapshotParams> for DecideWithSnapshotInput {
    fn from(value: DecideWithSnapshotParams) -> Self {
        DecideWithSnapshotInput {
            task: value.task,
            action: value.action,
            snapshot: value.snapshot.into(),
        }
    }
}

impl DecideWithSnapshotParams {
    fn auto_reflect_namespace(&self) -> Result<Option<Namespace>, AppError> {
        self.auto_reflect_namespace
            .as_deref()
            .map(|namespace| Namespace::parse(namespace.to_string()).map_err(AppError::from))
            .transpose()
    }
}

impl AutoReflectInput {
    pub fn from_decide(params: &DecideWithSnapshotParams) -> Result<Option<Self>, AppError> {
        params
            .auto_reflect_namespace()?
            .map(|namespace| Ok(Self::for_conflict(namespace, params.trigger_hints.clone())))
            .transpose()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReflectionDto {
    pub summary: String,
}

impl From<ReflectionDto> for Reflection {
    fn from(value: ReflectionDto) -> Self {
        Reflection::new(value.summary)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReflectionIdentityUpdateDto {
    pub canonical_claims: Vec<String>,
}

impl From<ReflectionIdentityUpdateDto> for ReflectionIdentityUpdate {
    fn from(value: ReflectionIdentityUpdateDto) -> Self {
        ReflectionIdentityUpdate::new(value.canonical_claims)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceQueryDto {
    #[serde(default)]
    pub owner: Option<OwnerDto>,
    #[serde(default)]
    pub kind: Option<EventKindDto>,
    #[serde(default)]
    pub limit: Option<usize>,
}

impl TryFrom<EvidenceQueryDto> for EvidenceQuery {
    type Error = AppError;

    fn try_from(value: EvidenceQueryDto) -> Result<Self, Self::Error> {
        if let Some(limit) = value.limit {
            i64::try_from(limit).map_err(|_| {
                AppError::InvalidParams(
                    "replacement evidence query limit exceeds the supported maximum".to_string(),
                )
            })?;
        }

        Ok(Self {
            owner: value.owner.map(Owner::from),
            kind: value.kind.map(EventKind::from),
            limit: value.limit,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RunReflectionParams {
    pub reflection: ReflectionDto,
    pub supersede_claim_id: String,
    pub replacement_claim: Option<ClaimDraftDto>,
    #[serde(default)]
    pub replacement_evidence_event_ids: Vec<String>,
    #[serde(default)]
    pub replacement_evidence_query: Option<EvidenceQueryDto>,
    #[serde(default)]
    pub identity_update: Option<ReflectionIdentityUpdateDto>,
    #[serde(default)]
    pub commitment_updates: Option<Vec<CommitmentDto>>,
}

impl TryFrom<RunReflectionParams> for ReflectionInput {
    type Error = AppError;

    fn try_from(value: RunReflectionParams) -> Result<Self, Self::Error> {
        let mut input = ReflectionInput::new(
            value.reflection.into(),
            value.supersede_claim_id,
            value
                .replacement_claim
                .map(ClaimDraft::try_from)
                .transpose()
                .map_err(AppError::from)?,
            value.replacement_evidence_event_ids,
        );

        if let Some(replacement_evidence_query) = value.replacement_evidence_query {
            input = input.with_replacement_evidence_query(replacement_evidence_query.try_into()?);
        }

        if let Some(identity_update) = value.identity_update {
            input = input.with_identity_update(identity_update.canonical_claims);
        }

        if let Some(commitment_updates) = value.commitment_updates {
            input = input.with_commitment_updates(
                commitment_updates
                    .into_iter()
                    .map(Commitment::from)
                    .collect(),
            );
        }

        Ok(input)
    }
}
