use agent_llm_mm::{
    application::{
        build_self_snapshot::BuildSelfSnapshotInput, decide_with_snapshot::DecideWithSnapshotInput,
        ingest_interaction::IngestInput, run_reflection::ReflectionInput,
    },
    domain::{
        claim::ClaimDraft,
        event::Event,
        reflection::Reflection,
        snapshot::{SelfSnapshot, SnapshotBudget},
        types::{EventKind, Mode, Owner},
    },
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
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub mode: ModeDto,
}

impl From<ClaimDraftDto> for ClaimDraft {
    fn from(value: ClaimDraftDto) -> Self {
        ClaimDraft::new(
            value.owner.into(),
            value.subject,
            value.predicate,
            value.object,
            value.mode.into(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IngestInteractionParams {
    pub event: EventDto,
    pub claim_drafts: Vec<ClaimDraftDto>,
    pub episode_reference: Option<String>,
}

impl From<IngestInteractionParams> for IngestInput {
    fn from(value: IngestInteractionParams) -> Self {
        IngestInput::new(
            value.event.into(),
            value.claim_drafts.into_iter().map(Into::into).collect(),
            value.episode_reference,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BuildSelfSnapshotParams {
    pub budget: usize,
}

impl From<BuildSelfSnapshotParams> for BuildSelfSnapshotInput {
    fn from(value: BuildSelfSnapshotParams) -> Self {
        BuildSelfSnapshotInput {
            budget: SnapshotBudget::new(value.budget),
        }
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
pub struct RunReflectionParams {
    pub reflection: ReflectionDto,
    pub supersede_claim_id: String,
    pub replacement_claim: Option<ClaimDraftDto>,
}

impl From<RunReflectionParams> for ReflectionInput {
    fn from(value: RunReflectionParams) -> Self {
        ReflectionInput::new(
            value.reflection.into(),
            value.supersede_claim_id,
            value.replacement_claim.map(Into::into),
        )
    }
}
