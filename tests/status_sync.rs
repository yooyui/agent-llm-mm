use agent_llm_mm::support::status_sync::{
    CargoTestList, DocumentTestTotal, RealityGateReport, StatusSyncReport, TEST_TOTAL_DOCUMENTS,
};

#[test]
fn documented_cargo_test_totals_match_current_test_list() {
    let cargo_tests = CargoTestList::parse(
        "\
alpha_case: test
beta_case: test
gamma_case: test
",
    );
    let documented_totals = TEST_TOTAL_DOCUMENTS
        .iter()
        .map(|path| {
            let contents = format!("- `cargo test` 全量通过，共 {} 个测试", cargo_tests.total);
            DocumentTestTotal::parse(*path, &contents).unwrap()
        })
        .collect::<Vec<_>>();

    let report = StatusSyncReport::from_totals(cargo_tests.total, documented_totals);

    assert!(
        report.is_in_sync(),
        "cargo test total drift detected:\n{}",
        report.format_mismatches()
    );
}

#[test]
fn monitored_documents_include_local_mcp_integration_entrypoint() {
    assert!(
        TEST_TOTAL_DOCUMENTS.contains(&"docs/local-mcp-integration-2026-03-26.md"),
        "local MCP integration docs carry fresh verification totals and must be part of drift checks"
    );
}

#[test]
fn status_sync_report_flags_documented_total_drift() {
    let report = StatusSyncReport::from_totals(
        256,
        vec![DocumentTestTotal {
            path: "README.md".to_string(),
            total: 255,
        }],
    );

    assert!(!report.is_in_sync());
    assert_eq!(
        report.format_mismatches(),
        "- README.md: documented 255, actual 256"
    );
}

#[test]
fn reality_gate_report_flags_implemented_plan_when_gate_is_still_partial() {
    let plan = "- [x] **P1.3 Plan/status synchronization v2**\n";
    let gates = "| `P1` | Plan/status synchronization | `partial` | still manual | keep gate open | `cargo test --test status_sync -v` |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert!(!report.is_in_sync());
    assert_eq!(report.contradictions.len(), 1);
    assert_eq!(
        report.contradictions[0].workstream,
        "Plan/status synchronization"
    );
    assert_eq!(report.contradictions[0].reality_status, "partial");
    assert!(
        report
            .format_contradictions()
            .contains("plan marks implemented but reality gate is partial")
    );
}

#[test]
fn reality_gate_report_flags_completed_plan_without_matching_reality_row() {
    let plan = "- [x] **P2.3 Richer episode semantics first slice**\n";
    let gates = "| `P2` | Structured decision protocol | `implemented` | code tests docs aligned | keep compatible | `cargo test --test decision_flow -v` |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert!(!report.is_in_sync());
    assert_eq!(report.missing_gates.len(), 1);
    assert_eq!(
        report.missing_gates[0].workstream,
        "Richer episode semantics first slice"
    );
    assert!(
        report
            .format_contradictions()
            .contains("no matching reality gate row exists")
    );
}

#[test]
fn reality_gate_report_allows_completed_plan_when_gate_is_implemented() {
    let plan = "- [x] **P1.1 Product readiness gate checker**\n";
    let gates = "| `P1` | Product readiness gate checker | `implemented` | code tests docs aligned | keep using checker | `cargo test --test product_readiness -v` |\n";

    let report = RealityGateReport::from_contents(plan, gates);

    assert!(
        report.is_in_sync(),
        "implemented gates should not be reported as contradictions"
    );
}
