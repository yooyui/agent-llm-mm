use agent_llm_mm::support::status_sync::{
    CargoTestList, DocumentTestTotal, StatusSyncReport, TEST_TOTAL_DOCUMENTS,
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
