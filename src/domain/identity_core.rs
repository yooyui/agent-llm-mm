use crate::domain::types::Mode;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IdentityCore {
    pub canonical_claims: Vec<String>,
}

impl IdentityCore {
    pub fn new(canonical_claims: Vec<String>) -> Self {
        Self { canonical_claims }
    }
}

pub fn allow_direct_ingest_update(mode: Mode) -> bool {
    match mode {
        Mode::Draft => false,
        Mode::Observed | Mode::Said | Mode::Acted | Mode::Inferred => false,
    }
}
