pub fn conflicts_with_commitment(action: &str, rule: &str) -> bool {
    rule == "forbid:write_identity_core_directly" && action == "write_identity_core_directly"
}
