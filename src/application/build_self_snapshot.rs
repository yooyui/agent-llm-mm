use crate::{
    domain::{
        rules::snapshot_builder::build_snapshot,
        snapshot::{SelfSnapshot, SnapshotBudget, SnapshotRequest},
    },
    error::AppError,
    ports::{ClaimStatus, ClaimStore, CommitmentStore, EpisodeStore, EventStore, IdentityStore},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BuildSelfSnapshotInput {
    pub budget: SnapshotBudget,
}

impl BuildSelfSnapshotInput {
    pub fn for_revision_window(evidence_window_len: usize) -> Self {
        Self {
            budget: SnapshotBudget::new(evidence_window_len.max(1)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BuildSelfSnapshotResult {
    pub snapshot: SelfSnapshot,
}

pub async fn execute<D>(
    deps: &D,
    input: BuildSelfSnapshotInput,
) -> Result<BuildSelfSnapshotResult, AppError>
where
    D: IdentityStore + CommitmentStore + ClaimStore + EventStore + EpisodeStore + Sync,
{
    let identity = deps.load_identity().await?;
    let commitments = deps
        .list_commitments()
        .await?
        .into_iter()
        .map(|commitment| commitment.description().to_string())
        .collect();
    let claims = deps
        .list_active_claims()
        .await?
        .into_iter()
        .filter(|claim| claim.status == ClaimStatus::Active)
        .map(|claim| claim.snapshot_value())
        .collect();
    let request = SnapshotRequest {
        identity: identity.canonical_claims().to_vec(),
        commitments,
        claims,
        evidence: deps.list_event_references().await?,
        episodes: deps.list_episode_references().await?,
        budget: input.budget,
    };

    Ok(BuildSelfSnapshotResult {
        snapshot: build_snapshot(request)?,
    })
}
