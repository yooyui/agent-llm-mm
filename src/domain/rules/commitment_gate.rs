#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GateResult {
    pub blocked: bool,
}

pub fn gate_decision(action: &str, commitments: &[String]) -> GateResult {
    let blocked = commitments
        .iter()
        .any(|rule| rule == "forbid:write_identity_core_directly")
        && action == "write_identity_core_directly";

    GateResult { blocked }
}
