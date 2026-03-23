#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceLink {
    pub source_event_id: String,
    pub target_claim_id: String,
}

impl EvidenceLink {
    pub fn new(source_event_id: impl Into<String>, target_claim_id: impl Into<String>) -> Self {
        Self {
            source_event_id: source_event_id.into(),
            target_claim_id: target_claim_id.into(),
        }
    }
}
