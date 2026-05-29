use std::collections::HashSet;

use crate::{domain::self_revision::SELF_REVISION_DURABLE_WRITE_PATH, error::AppError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpisodeProjectionInput {
    pub episode_reference: String,
    pub episode_event_ids: Vec<String>,
    pub objective: Option<String>,
    pub outcome: Option<String>,
    pub lesson: Option<String>,
    pub linked_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EpisodeSummaryProjection {
    pub episode_reference: String,
    pub objective: Option<String>,
    pub outcome: Option<String>,
    pub lesson: Option<String>,
    pub linked_evidence_ids: Vec<String>,
    pub event_count: usize,
    pub writes_performed: bool,
    pub durable_self_model_write_path: String,
    pub identity_or_commitment_updates: Vec<String>,
}

pub fn build_episode_summary_projection(
    input: EpisodeProjectionInput,
) -> Result<EpisodeSummaryProjection, AppError> {
    if input.episode_reference.trim().is_empty() {
        return Err(AppError::InvalidParams(
            "episode projection requires an episode reference".to_string(),
        ));
    }
    let episode_event_ids = dedupe(input.episode_event_ids);
    let event_set = episode_event_ids.iter().cloned().collect::<HashSet<_>>();
    if let Some(outside_id) = input
        .linked_evidence_ids
        .iter()
        .find(|event_id| !event_set.contains(*event_id))
    {
        return Err(AppError::InvalidParams(format!(
            "linked evidence {outside_id} is outside the episode event set"
        )));
    }

    Ok(EpisodeSummaryProjection {
        episode_reference: input.episode_reference,
        objective: input.objective,
        outcome: input.outcome,
        lesson: input.lesson,
        linked_evidence_ids: dedupe(input.linked_evidence_ids),
        event_count: episode_event_ids.len(),
        writes_performed: false,
        durable_self_model_write_path: SELF_REVISION_DURABLE_WRITE_PATH.to_string(),
        identity_or_commitment_updates: Vec::new(),
    })
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !deduped.contains(&value) {
            deduped.push(value);
        }
    }
    deduped
}
