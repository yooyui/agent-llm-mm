use crate::domain::{
    DomainError,
    snapshot::{SelfSnapshot, SnapshotRequest},
};

pub fn build_snapshot(input: SnapshotRequest) -> Result<SelfSnapshot, DomainError> {
    let input = input.validate()?;
    let evidence = input
        .evidence
        .iter()
        .take(input.budget.max(1))
        .cloned()
        .collect::<Vec<_>>();

    if evidence.is_empty() {
        return Err(DomainError::InsufficientEvidence);
    }

    Ok(SelfSnapshot {
        identity: input.identity,
        commitments: input.commitments,
        claims: input.claims,
        evidence,
        episodes: input.episodes,
    })
}
