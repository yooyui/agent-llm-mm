use crate::{
    domain::{
        claim::ClaimDraft,
        reflection::Reflection,
        rules::reflection_policy::{ReflectionDecision, ReflectionTrigger, classify_reflection},
    },
    error::AppError,
    ports::{
        ClaimStatus, Clock, EventStore, IdGenerator, ReflectionTransactionRunner, StoredClaim,
        StoredReflection,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReflectionInput {
    reflection: Reflection,
    supersede_claim_id: String,
    replacement_claim: Option<ClaimDraft>,
    replacement_evidence_event_ids: Vec<String>,
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
        }
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
    } = input;
    let reflection_id = deps.next_id().await?;
    let recorded_at = deps.now().await?;
    let mut transaction = deps.begin_reflection_transaction().await?;
    let decision = classify_reflection(match replacement_claim {
        Some(_) => ReflectionTrigger::Failure,
        None => ReflectionTrigger::Conflict,
    });
    let replacement_claim_id = match (decision, replacement_claim) {
        (ReflectionDecision::SupersedeWithReplacement, Some(claim)) => {
            for event_id in &replacement_evidence_event_ids {
                if !deps.has_event(event_id).await? {
                    return Err(AppError::InvalidParams(format!(
                        "unknown replacement evidence event id: {event_id}"
                    )));
                }
            }
            claim.validate(replacement_evidence_event_ids.len())?;
            let claim_id = format!("{reflection_id}:replacement");
            transaction
                .upsert_claim(StoredClaim::new(
                    claim_id.clone(),
                    claim,
                    ClaimStatus::Active,
                ))
                .await?;
            for event_id in &replacement_evidence_event_ids {
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

    transaction
        .append_reflection(StoredReflection::new(
            reflection_id.clone(),
            recorded_at,
            reflection,
            Some(supersede_claim_id.clone()),
            replacement_claim_id.clone(),
        ))
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
