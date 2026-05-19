use crate::{
    domain::{rules::commitment_gate::gate_decision, snapshot::SelfSnapshot},
    error::AppError,
    ports::{ModelDecision, ModelDecisionRequest, ModelPort},
};

const DECISION_PROTOCOL_VERSION: u32 = 1;
const COMMITMENT_GATE_NAME: &str = "commitment_gate";
const COMMITMENT_GATE_BLOCKED_REASON: &str = "commitment_gate_blocked_action";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecideWithSnapshotInput {
    pub task: String,
    pub action: String,
    pub snapshot: SelfSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecideWithSnapshotResult {
    pub blocked: bool,
    pub decision: Option<ModelDecision>,
    pub protocol_version: u32,
    pub status: String,
    pub reason: Option<String>,
    pub gate: DecisionGateMetadata,
}

impl DecideWithSnapshotResult {
    fn blocked_by_commitment_gate() -> Self {
        Self {
            blocked: true,
            decision: None,
            protocol_version: DECISION_PROTOCOL_VERSION,
            status: "blocked".to_string(),
            reason: Some(COMMITMENT_GATE_BLOCKED_REASON.to_string()),
            gate: DecisionGateMetadata::blocked(COMMITMENT_GATE_BLOCKED_REASON),
        }
    }

    fn model_decision(decision: ModelDecision) -> Self {
        Self {
            blocked: false,
            decision: Some(decision),
            protocol_version: DECISION_PROTOCOL_VERSION,
            status: "model_decision".to_string(),
            reason: None,
            gate: DecisionGateMetadata::passed(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecisionGateMetadata {
    pub name: String,
    pub blocked: bool,
    pub reason: Option<String>,
}

impl DecisionGateMetadata {
    fn blocked(reason: &str) -> Self {
        Self {
            name: COMMITMENT_GATE_NAME.to_string(),
            blocked: true,
            reason: Some(reason.to_string()),
        }
    }

    fn passed() -> Self {
        Self {
            name: COMMITMENT_GATE_NAME.to_string(),
            blocked: false,
            reason: None,
        }
    }
}

pub async fn execute<D>(
    deps: &D,
    input: DecideWithSnapshotInput,
) -> Result<DecideWithSnapshotResult, AppError>
where
    D: ModelPort + Sync,
{
    let gate = gate_decision(&input.action, &input.snapshot.commitments);
    if gate.blocked {
        return Ok(DecideWithSnapshotResult::blocked_by_commitment_gate());
    }

    let decision = deps
        .decide(ModelDecisionRequest::new(
            input.task,
            input.action,
            input.snapshot,
        ))
        .await?;

    Ok(DecideWithSnapshotResult::model_decision(decision))
}
