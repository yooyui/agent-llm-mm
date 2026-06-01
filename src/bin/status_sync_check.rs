use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

use agent_llm_mm::support::status_sync::{
    CargoTestList, DocumentTestTotal, PLAN_STATUS_DOCUMENT, REALITY_GATES_DOCUMENT,
    RealityGateReport, StatusSyncReport, TEST_TOTAL_DOCUMENTS,
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let list = CargoTestList::parse(&stdout);
    let documents = TEST_TOTAL_DOCUMENTS
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .map(|contents| (*path, contents))
                .map_err(anyhow::Error::from)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let parsed_documents = documents
        .iter()
        .map(|(path, contents)| {
            DocumentTestTotal::parse(*path, contents).ok_or_else(|| {
                agent_llm_mm::support::status_sync::MissingDocumentTotal {
                    path: (*path).to_string(),
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let actual_suite_counts = integration_suite_counts_from_test_binaries(&stderr)?;
    let report = StatusSyncReport::from_counts(list.total, actual_suite_counts, parsed_documents);

    let plan_contents = fs::read_to_string(PLAN_STATUS_DOCUMENT)?;
    let reality_gate_contents = fs::read_to_string(REALITY_GATES_DOCUMENT)?;
    let reality_report = RealityGateReport::from_contents(&plan_contents, &reality_gate_contents);

    if report.is_in_sync() && reality_report.is_in_sync() {
        println!(
            "status sync ok: documented cargo test totals and suite counts match {}; plan/status reality gates are in sync",
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

fn integration_suite_counts_from_test_binaries(
    cargo_test_stderr: &str,
) -> anyhow::Result<BTreeMap<String, usize>> {
    test_binaries_from_cargo_stderr(cargo_test_stderr)
        .into_iter()
        .map(|(suite, path)| {
            let output = Command::new(&path)
                .args(["--list", "--format", "terse"])
                .output()?;
            if !output.status.success() {
                anyhow::bail!(
                    "test binary list failed for {suite} at {}:\n{}",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok((
                suite,
                CargoTestList::parse(&String::from_utf8_lossy(&output.stdout)).total,
            ))
        })
        .collect()
}

fn test_binaries_from_cargo_stderr(cargo_test_stderr: &str) -> BTreeMap<String, PathBuf> {
    cargo_test_stderr
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.strip_prefix("Running tests/")?;
            let (suite_path, rest) = rest.split_once(" (")?;
            let suite = suite_path.strip_suffix(".rs")?.to_string();
            let path = rest.strip_suffix(')')?;
            Some((suite, PathBuf::from(path)))
        })
        .collect()
}
