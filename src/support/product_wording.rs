#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClaimGateState {
    pub beta: bool,
    pub ga: bool,
    pub production_ready: bool,
    pub remote_write_admin: bool,
    pub remote_team_service: bool,
    pub complete_self_governance: bool,
    pub physics_informed_runtime: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductClaimGuardInput {
    pub text: String,
    pub gate_state: ClaimGateState,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProductClaimGuardReport {
    pub allowed: bool,
    pub violations: Vec<ProductClaimViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProductClaimViolation {
    pub claim: &'static str,
    pub matched_text: String,
    pub required_gate: &'static str,
}

pub fn check_product_claims(input: ProductClaimGuardInput) -> ProductClaimGuardReport {
    let mut violations = Vec::new();
    let text = input.text.to_ascii_lowercase();
    let gate_state = input.gate_state;

    push_if_claimed(
        &mut violations,
        &text,
        &["beta"],
        gate_state.beta,
        "beta",
        "beta_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &["ga", "general availability"],
        gate_state.ga,
        "ga",
        "ga_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &["production-ready", "production ready"],
        gate_state.production_ready,
        "production_ready",
        "production_readiness_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &["remote write admin"],
        gate_state.remote_write_admin,
        "remote_write_admin",
        "remote_write_admin_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &[
            "remote team service",
            "remote-team service",
            "remote/team service",
            "remote team",
            "remote-team",
            "team service",
        ],
        gate_state.remote_team_service,
        "remote_team_service",
        "remote_team_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &[
            "complete self-governance",
            "complete self governance",
            "complete self-governing",
            "full self-governance",
        ],
        gate_state.complete_self_governance,
        "complete_self_governance",
        "self_governance_gate",
    );
    push_if_claimed(
        &mut violations,
        &text,
        &[
            "physics-informed runtime",
            "physics informed runtime",
            "physics-informed-runtime",
            "physics solver",
            "physics-solver",
            "physics-informed solver",
            "physics informed solver",
            "physics-informed-solver",
            "physics informed controller",
            "physics-informed controller",
            "physics-informed-controller",
            "constraint solver",
            "constraint-solver",
            "constraint optimizer",
            "constraint-optimizer",
            "physical controller",
            "physical-controller",
            "scientific validation",
            "scientific-validation",
        ],
        gate_state.physics_informed_runtime,
        "physics_informed_runtime",
        "physics_informed_runtime_gate",
    );

    ProductClaimGuardReport {
        allowed: violations.is_empty(),
        violations,
    }
}

fn push_if_claimed(
    violations: &mut Vec<ProductClaimViolation>,
    text: &str,
    patterns: &[&'static str],
    gate_satisfied: bool,
    claim: &'static str,
    required_gate: &'static str,
) {
    if gate_satisfied {
        return;
    }

    if let Some(pattern) = patterns
        .iter()
        .find(|pattern| pattern_matches(text, pattern))
    {
        violations.push(ProductClaimViolation {
            claim,
            matched_text: (*pattern).to_string(),
            required_gate,
        });
    }
}

fn pattern_matches(text: &str, pattern: &str) -> bool {
    if pattern == "ga" {
        return text
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .any(|token| token == "ga");
    }

    text.contains(pattern)
}
