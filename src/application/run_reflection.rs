use crate::{
    domain::{
        claim::ClaimDraft,
        commitment::Commitment,
        identity_core::IdentityCore,
        reflection::{Reflection, ReflectionIdentityUpdate},
        rules::reflection_policy::{ReflectionDecision, ReflectionTrigger, classify_reflection},
    },
    error::AppError,
    ports::{
        ClaimStatus, Clock, EventStore, EvidenceQuery, IdGenerator, ReflectionTransactionRunner,
        StoredClaim, StoredReflection,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReflectionInput {
    reflection: Reflection,
    supersede_claim_id: String,
    replacement_claim: Option<ClaimDraft>,
    replacement_evidence_event_ids: Vec<String>,
    replacement_evidence_query: Option<EvidenceQuery>,
    identity_update: Option<ReflectionIdentityUpdate>,
    commitment_updates: Option<Vec<Commitment>>,
}

impl ReflectionInput {
    pub fn new(
        reflection: Reflection,
        supersede_claim_id: impl Into<String>,
        replacement_claim: Option<ClaimDraft>,
        replacement_evidence_event_ids: Vec<String>,
    ) -> Self {
        Self {
            reflection,
            supersede_claim_id: supersede_claim_id.into(),
            replacement_claim,
            replacement_evidence_event_ids,
            replacement_evidence_query: None,
            identity_update: None,
            commitment_updates: None,
        }
    }

    pub fn with_replacement_evidence_query(
        mut self,
        replacement_evidence_query: EvidenceQuery,
    ) -> Self {
        self.replacement_evidence_query = Some(replacement_evidence_query);
        self
    }

    pub fn with_identity_update(mut self, canonical_claims: Vec<String>) -> Self {
        self.identity_update = Some(ReflectionIdentityUpdate::new(canonical_claims));
        self
    }

    pub fn with_commitment_updates(mut self, commitments: Vec<Commitment>) -> Self {
        self.commitment_updates = Some(commitments);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReflectionResult {
    pub reflection_id: String,
    pub replacement_claim_id: Option<String>,
}

pub async fn execute<D>(deps: &D, input: ReflectionInput) -> Result<ReflectionResult, AppError>
where
    D: ReflectionTransactionRunner + EventStore + IdGenerator + Clock + Sync,
{
    let ReflectionInput {
        reflection,
        supersede_claim_id,
        replacement_claim,
        replacement_evidence_event_ids,
        replacement_evidence_query,
        identity_update,
        commitment_updates,
    } = input;

    let reflection_id = deps.next_id().await?;
    let recorded_at = deps.now().await?;
    let decision = classify_reflection(if replacement_claim.is_some() {
        ReflectionTrigger::Failure
    } else {
        ReflectionTrigger::Conflict
    });
    let requires_supporting_evidence =
        replacement_claim.is_some() || identity_update.is_some() || commitment_updates.is_some();
    let supporting_evidence_event_ids = if requires_supporting_evidence {
        resolve_evidence_event_ids(
            deps,
            replacement_evidence_query,
            replacement_evidence_event_ids,
        )
        .await?
    } else {
        Vec::new()
    };

    if (identity_update.is_some() || commitment_updates.is_some())
        && supporting_evidence_event_ids.is_empty()
    {
        return Err(AppError::InvalidParams(
            "identity or commitment reflection updates require at least one resolved evidence event id"
                .to_string(),
        ));
    }

    if identity_update
        .as_ref()
        .is_some_and(|update| update.canonical_claims.is_empty())
    {
        return Err(AppError::InvalidParams(
            "identity reflection updates must include at least one canonical claim".to_string(),
        ));
    }

    for event_id in &supporting_evidence_event_ids {
        if !deps.has_event(event_id).await? {
            return Err(AppError::InvalidParams(format!(
                "unknown replacement evidence event id: {event_id}"
            )));
        }
    }

    let mut transaction = deps.begin_reflection_transaction().await?;
    let commitment_updates = if let Some(commitment_updates) = commitment_updates {
        let existing_commitments = transaction.load_commitments().await?;
        Some(preserve_baseline_commitments(
            commitment_updates,
            existing_commitments,
        ))
    } else {
        None
    };
    let replacement_claim_id = match (decision, replacement_claim) {
        (ReflectionDecision::SupersedeWithReplacement, Some(claim)) => {
            claim.validate(supporting_evidence_event_ids.len())?;
            let claim_id = format!("{reflection_id}:replacement");
            transaction
                .upsert_claim(StoredClaim::new(
                    claim_id.clone(),
                    claim,
                    ClaimStatus::Active,
                ))
                .await?;
            for event_id in &supporting_evidence_event_ids {
                transaction
                    .link_evidence(claim_id.clone(), event_id.clone())
                    .await?;
            }
            Some(claim_id)
        }
        (ReflectionDecision::SupersedeWithReplacement, None) => {
            return Err(AppError::Message(
                "superseding reflections require a replacement claim".to_string(),
            ));
        }
        _ => None,
    };

    if let Some(identity_update) = &identity_update {
        let _ = transaction.load_identity().await?;
        transaction
            .replace_identity(IdentityCore::new(identity_update.canonical_claims.clone()))
            .await?;
    }

    if let Some(commitment_updates) = &commitment_updates {
        transaction
            .replace_commitments(commitment_updates.clone())
            .await?;
    }

    transaction
        .append_reflection(
            StoredReflection::new(
                reflection_id.clone(),
                recorded_at,
                reflection,
                Some(supersede_claim_id.clone()),
                replacement_claim_id.clone(),
            )
            .with_supporting_evidence_event_ids(supporting_evidence_event_ids)
            .with_requested_identity_update(identity_update)
            .with_requested_commitment_updates(commitment_updates),
        )
        .await?;

    match decision {
        ReflectionDecision::MarkDisputed => {
            transaction
                .update_claim_status(&supersede_claim_id, ClaimStatus::Disputed)
                .await?;
        }
        ReflectionDecision::SupersedeWithReplacement => {
            transaction
                .update_claim_status(&supersede_claim_id, ClaimStatus::Superseded)
                .await?;
        }
        ReflectionDecision::RecordOnly => {}
    }

    transaction.commit().await?;

    Ok(ReflectionResult {
        reflection_id,
        replacement_claim_id,
    })
}

fn preserve_baseline_commitments(
    mut requested_commitments: Vec<Commitment>,
    existing_commitments: Vec<Commitment>,
) -> Vec<Commitment> {
    for commitment in existing_commitments {
        if is_baseline_commitment(&commitment)
            && !requested_commitments
                .iter()
                .any(|candidate| candidate == &commitment)
        {
            requested_commitments.push(commitment);
        }
    }

    requested_commitments
}

fn is_baseline_commitment(commitment: &Commitment) -> bool {
    commitment.description() == "forbid:write_identity_core_directly"
}

async fn resolve_evidence_event_ids<D>(
    deps: &D,
    query: Option<EvidenceQuery>,
    explicit: Vec<String>,
) -> Result<Vec<String>, AppError>
where
    D: EventStore + Sync,
{
    let mut evidence_event_ids = explicit;

    if let Some(query) = query {
        let mut queried_ids = deps.query_evidence_event_ids(query).await?;
        if queried_ids.is_empty() && evidence_event_ids.is_empty() {
            return Err(AppError::InvalidParams(
                "no replacement evidence found for the provided query".to_string(),
            ));
        }
        evidence_event_ids.append(&mut queried_ids);
    }

    let mut deduped = Vec::new();
    for event_id in evidence_event_ids {
        if !deduped.contains(&event_id) {
            deduped.push(event_id);
        }
    }

    Ok(deduped)
}
