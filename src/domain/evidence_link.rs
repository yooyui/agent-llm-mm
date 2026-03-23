#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EvidenceLink {
    source_event_id: String,
    target_claim_id: String,
}

impl EvidenceLink {
    pub fn new(source_event_id: impl Into<String>, target_claim_id: impl Into<String>) -> Self {
        Self {
            source_event_id: source_event_id.into(),
            target_claim_id: target_claim_id.into(),
        }
    }

    pub fn source_event_id(&self) -> &str {
        &self.source_event_id
    }

    pub fn target_claim_id(&self) -> &str {
        &self.target_claim_id
    }
}
