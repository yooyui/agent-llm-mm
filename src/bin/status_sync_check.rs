use std::{fs, process::Command};

use agent_llm_mm::support::status_sync::{
    CargoTestList, TEST_TOTAL_DOCUMENTS, report_from_document_contents,
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

    if report.is_in_sync() {
        println!(
            "status sync ok: documented cargo test totals match {}",
            report.actual_total
        );
        return Ok(());
    }

    anyhow::bail!(
        "status sync drift detected:\n{}",
        report.format_mismatches()
    );
}
