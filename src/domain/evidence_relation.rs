use std::collections::HashSet;

use crate::error::AppError;

pub const EVIDENCE_RELATION_PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRelationInput {
    pub trigger_window_event_ids: Vec<String>,
    pub selected_evidence_event_ids: Vec<String>,
    pub selection_basis: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EvidenceRelationReport {
    pub protocol_version: u32,
    pub trigger_window_size: usize,
    pub selected_count: usize,
    pub no_widening_policy: &'static str,
    pub relations: Vec<EvidenceRelation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceRelation {
    pub event_id: String,
    pub window_rank: usize,
    pub selected: bool,
    pub selection_basis: Option<String>,
}

pub fn build_evidence_relation_report(
    input: EvidenceRelationInput,
) -> Result<EvidenceRelationReport, AppError> {
    let selected_ids = dedupe(input.selected_evidence_event_ids);
    let trigger_window_ids = dedupe(input.trigger_window_event_ids);
    let trigger_window_set = trigger_window_ids.iter().cloned().collect::<HashSet<_>>();

    if let Some(outside_id) = selected_ids
        .iter()
        .find(|event_id| !trigger_window_set.contains(*event_id))
    {
        return Err(AppError::InvalidParams(format!(
            "selected evidence {outside_id} is outside the trigger window"
        )));
    }

    let relations = trigger_window_ids
        .iter()
        .enumerate()
        .map(|(index, event_id)| {
            let selected = selected_ids.contains(event_id);
            EvidenceRelation {
                event_id: event_id.clone(),
                window_rank: index + 1,
                selected,
                selection_basis: selected.then(|| input.selection_basis.clone()).flatten(),
            }
        })
        .collect::<Vec<_>>();

    Ok(EvidenceRelationReport {
        protocol_version: EVIDENCE_RELATION_PROTOCOL_VERSION,
        trigger_window_size: trigger_window_ids.len(),
        selected_count: selected_ids.len(),
        no_widening_policy: "selected_subset_of_trigger_window",
        relations,
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
