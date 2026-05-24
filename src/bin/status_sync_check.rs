use std::{fs, process::Command};

use agent_llm_mm::support::status_sync::{
    CargoTestList, PLAN_STATUS_DOCUMENT, REALITY_GATES_DOCUMENT, RealityGateReport,
    TEST_TOTAL_DOCUMENTS, report_from_document_contents,
};

fn main() -> anyhow::Result<()> {
    let output = Command::new("cargo")
        .args(["test", "--", "--list", "--format", "terse"])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo test list failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let list = CargoTestList::parse(&String::from_utf8_lossy(&output.stdout));
    let documents = TEST_TOTAL_DOCUMENTS
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .map(|contents| (*path, contents))
                .map_err(anyhow::Error::from)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let borrowed_documents = documents
        .iter()
        .map(|(path, contents)| (*path, contents.as_str()));
    let report = report_from_document_contents(list.total, borrowed_documents)?;

    let plan_contents = fs::read_to_string(PLAN_STATUS_DOCUMENT)?;
    let reality_gate_contents = fs::read_to_string(REALITY_GATES_DOCUMENT)?;
    let reality_report = RealityGateReport::from_contents(&plan_contents, &reality_gate_contents);

    if report.is_in_sync() && reality_report.is_in_sync() {
        println!(
            "status sync ok: documented cargo test totals match {}; plan/status reality gates are in sync",
            report.actual_total
        );
        return Ok(());
    }

    let mut sections = Vec::new();
    if !report.is_in_sync() {
        sections.push(format!(
            "cargo test total drift:\n{}",
            report.format_mismatches()
        ));
    }
    if !reality_report.is_in_sync() {
        sections.push(format!(
            "plan/status reality gate drift:\n{}",
            reality_report.format_contradictions()
        ));
    }

    anyhow::bail!("status sync drift detected:\n{}", sections.join("\n\n"));
}
