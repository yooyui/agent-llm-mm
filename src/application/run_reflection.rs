use crate::{
    domain::{claim::ClaimDraft, reflection::Reflection},
    error::AppError,
    ports::{
        ClaimStatus, ClaimStore, Clock, IdGenerator, ReflectionStore, StoredClaim, StoredReflection,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReflectionInput {
    reflection: Reflection,
    supersede_claim_id: String,
    replacement_claim: Option<ClaimDraft>,
}

impl ReflectionInput {
    pub fn new(
        reflection: Reflection,
        supersede_claim_id: impl Into<String>,
        replacement_claim: Option<ClaimDraft>,
    ) -> Self {
        Self {
            reflection,
            supersede_claim_id: supersede_claim_id.into(),
            replacement_claim,
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
    D: ReflectionStore + ClaimStore + IdGenerator + Clock + Sync,
{
    let reflection_id = deps.next_id().await?;
    let recorded_at = deps.now().await?;
    let replacement_claim_id = match input.replacement_claim {
        Some(claim) => {
            claim.validate(1)?;
            let claim_id = format!("{reflection_id}:replacement");
            deps.upsert_claim(StoredClaim::new(
                claim_id.clone(),
                claim,
                ClaimStatus::Active,
            ))
            .await?;
            Some(claim_id)
        }
        None => None,
    };

    deps.append_reflection(StoredReflection::new(
        reflection_id.clone(),
        recorded_at,
        input.reflection,
        Some(input.supersede_claim_id.clone()),
        replacement_claim_id.clone(),
    ))
    .await?;
    deps.update_claim_status(&input.supersede_claim_id, ClaimStatus::Superseded)
        .await?;

    Ok(ReflectionResult {
        reflection_id,
        replacement_claim_id,
    })
}
