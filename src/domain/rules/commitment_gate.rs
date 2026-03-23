use crate::domain::rules::conflict::conflicts_with_commitment;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GateResult {
    pub blocked: bool,
}

pub fn gate_decision(action: &str, commitments: &[String]) -> GateResult {
    let blocked = commitments
        .iter()
        .any(|rule| conflicts_with_commitment(action, rule));

    GateResult { blocked }
}
