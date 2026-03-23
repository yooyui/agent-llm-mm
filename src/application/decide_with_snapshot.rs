use crate::{
    domain::{rules::commitment_gate::gate_decision, snapshot::SelfSnapshot},
    error::AppError,
    ports::{ModelDecision, ModelDecisionRequest, ModelPort},
};

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
        return Ok(DecideWithSnapshotResult {
            blocked: true,
            decision: None,
        });
    }

    let decision = deps
        .decide(ModelDecisionRequest::new(
            input.task,
            input.action,
            input.snapshot,
        ))
        .await?;

    Ok(DecideWithSnapshotResult {
        blocked: false,
        decision: Some(decision),
    })
}
