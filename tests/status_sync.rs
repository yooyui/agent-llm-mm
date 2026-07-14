use std::fs;

use agent_llm_mm::support::status_sync::{
    PLAN_STATUS_DOCUMENT, REALITY_GATES_DOCUMENT, RealityGateReport,
};

#[test]
fn active_plan_and_reality_gate_documents_are_in_sync() {
    assert_eq!(
        PLAN_STATUS_DOCUMENT, "docs/plans/2026-07-10-product-replan.md",
        "status sync must read the only active execution plan"
    );

    let plan = fs::read_to_string(PLAN_STATUS_DOCUMENT).expect("active plan document");
    let gates = fs::read_to_string(REALITY_GATES_DOCUMENT).expect("reality gate document");
    let wrapper = fs::read_to_string("scripts/status-sync-check.sh").expect("status sync wrapper");
    let report = RealityGateReport::from_contents(&plan, &gates);

    assert!(wrapper.contains("rustc --edition=2024"));
    assert!(!wrapper.contains("cargo test"));
    assert!(!wrapper.contains("cargo run"));
    assert!(
        report
            .completed_plan_items
            .contains(&"M0.1 Mainline and repository hygiene".to_string()),
        "active plan must expose the completed M0.1 milestone as a checkbox"
    );
    assert!(
        report
            .completed_plan_items
            .contains(&"M0.1.1 Test and toolchain slimming".to_string()),
        "active plan must expose the completed M0.1.1 milestone as a checkbox"
    );
    assert!(
        report
            .completed_plan_items
            .contains(&"M0.2 Scoped Snapshot v2".to_string()),
        "active plan must expose the completed M0.2 milestone as a checkbox"
    );
    assert!(
        report
            .completed_plan_items
            .contains(&"M0.3.1 Trusted decision commitments and dual gate".to_string()),
        "active plan must expose the completed M0.3.1 slice as a checkbox"
    );

    assert!(
        report.is_in_sync(),
        "plan/status reality gate drift detected:\n{}",
        report.format_contradictions()
    );
}

#[test]
fn reality_gate_report_rejects_vacuous_plan_without_completed_items() {
    let plan = "- [ ] M0.2 Scoped Snapshot v2\n";
    let gates = "| `M0` | M0.2 Scoped Snapshot v2 | `partial` | open | keep open | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert!(report.completed_plan_items.is_empty());
    assert!(!report.is_in_sync());
    assert!(report.format_contradictions().contains("would be vacuous"));
}

#[test]
fn reality_gate_report_flags_implemented_plan_when_gate_is_still_partial() {
    let plan = "- [x] **P1.3 Plan/status synchronization v2**\n";
    let gates = "| `P1` | Plan/status synchronization | `partial` | still manual | keep gate open | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert_eq!(report.contradictions.len(), 1);
    assert_eq!(report.contradictions[0].reality_status, "partial");
    assert!(!report.is_in_sync());
}

#[test]
fn reality_gate_report_flags_completed_plan_without_matching_reality_row() {
    let plan = "- [x] **P2.3 Richer episode semantics first slice**\n";
    let gates =
        "| `P2` | Structured decision protocol | `implemented` | aligned | keep | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert_eq!(report.missing_gates.len(), 1);
    assert_eq!(
        report.missing_gates[0].workstream,
        "Richer episode semantics first slice"
    );
}

#[test]
fn reality_gate_report_flags_implemented_unmerged_as_incomplete() {
    let plan = "- [x] **P3.4 Product wording guard**\n";
    let gates = "| `P3` | Product wording guard | `implemented-unmerged` | branch only | keep blocked | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert_eq!(report.contradictions.len(), 1);
    assert_eq!(
        report.contradictions[0].reality_status,
        "implemented-unmerged"
    );
}

#[test]
fn reality_gate_report_flags_blocked_claim_as_incomplete() {
    let plan = "- [x] **P3.5 Physics-informed runtime claim guard**\n";
    let gates = "| `P3` | Physics-informed runtime claim guard | `blocked claim` | wording only | keep blocked | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert_eq!(report.contradictions.len(), 1);
    assert_eq!(report.contradictions[0].reality_status, "blocked claim");
}

#[test]
fn reality_gate_report_allows_completed_plan_when_gate_is_implemented() {
    let plan = "- [x] **P1.1 Product readiness gate checker**\n";
    let gates =
        "| `P1` | Product readiness gate checker | `implemented` | aligned | keep | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert!(report.is_in_sync());
}

#[test]
fn reality_gate_report_parses_unbolded_completed_plan_items() {
    let plan = "- [x] P1.1 Product readiness gate checker\n";
    let gates =
        "| `P1` | Product readiness gate checker | `implemented` | aligned | keep | check |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert_eq!(
        report.completed_plan_items,
        vec!["Product readiness gate checker"]
    );
    assert!(report.is_in_sync());
}
