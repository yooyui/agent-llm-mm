pub const TEST_TOTAL_DOCUMENTS: &[&str] = &[
    "README.md",
    "docs/testing-guide-2026-03-24.md",
    "docs/local-mcp-integration-2026-03-26.md",
    "docs/project-status.md",
    "docs/progress-tracker.md",
    "docs/product/follow-up-reality-gates.md",
];

pub const PLAN_STATUS_DOCUMENT: &str =
    "docs/superpowers/plans/2026-05-24-p1-p2-p3-product-completion-plan.md";
pub const REALITY_GATES_DOCUMENT: &str = "docs/product/follow-up-reality-gates.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CargoTestList {
    pub total: usize,
}

impl CargoTestList {
    pub fn parse(output: &str) -> Self {
        let total = output
            .lines()
            .filter(|line| line.trim_end().ends_with(": test"))
            .count();

        Self { total }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentTestTotal {
    pub path: String,
    pub total: usize,
}

impl DocumentTestTotal {
    pub fn parse(path: impl Into<String>, contents: &str) -> Option<Self> {
        parse_cargo_test_total(contents).map(|total| Self {
            path: path.into(),
            total,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSyncMismatch {
    pub path: String,
    pub documented_total: usize,
    pub actual_total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSyncReport {
    pub actual_total: usize,
    pub documented_totals: Vec<DocumentTestTotal>,
    pub mismatches: Vec<StatusSyncMismatch>,
}

impl StatusSyncReport {
    pub fn from_totals(actual_total: usize, documented_totals: Vec<DocumentTestTotal>) -> Self {
        let mismatches = documented_totals
            .iter()
            .filter(|document| document.total != actual_total)
            .map(|document| StatusSyncMismatch {
                path: document.path.clone(),
                documented_total: document.total,
                actual_total,
            })
            .collect();

        Self {
            actual_total,
            documented_totals,
            mismatches,
        }
    }

    pub fn is_in_sync(&self) -> bool {
        self.mismatches.is_empty()
    }

    pub fn format_mismatches(&self) -> String {
        if self.mismatches.is_empty() {
            return format!(
                "all documented cargo test totals match {}",
                self.actual_total
            );
        }

        self.mismatches
            .iter()
            .map(|mismatch| {
                format!(
                    "- {}: documented {}, actual {}",
                    mismatch.path, mismatch.documented_total, mismatch.actual_total
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn report_from_document_contents<'a>(
    actual_total: usize,
    documents: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<StatusSyncReport, MissingDocumentTotal> {
    let mut documented_totals = Vec::new();

    for (path, contents) in documents {
        let document =
            DocumentTestTotal::parse(path, contents).ok_or_else(|| MissingDocumentTotal {
                path: path.to_string(),
            })?;
        documented_totals.push(document);
    }

    Ok(StatusSyncReport::from_totals(
        actual_total,
        documented_totals,
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealityGateContradiction {
    pub workstream: String,
    pub reality_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingRealityGate {
    pub workstream: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealityGateReport {
    pub completed_plan_items: Vec<String>,
    pub reality_gates: Vec<RealityGateDocumentRow>,
    pub contradictions: Vec<RealityGateContradiction>,
    pub missing_gates: Vec<MissingRealityGate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealityGateDocumentRow {
    pub workstream: String,
    pub status: String,
}

impl RealityGateReport {
    pub fn from_contents(plan_contents: &str, reality_gate_contents: &str) -> Self {
        let completed_plan_items = parse_completed_plan_items(plan_contents);
        let reality_gates = parse_reality_gate_rows(reality_gate_contents);
        let mut contradictions = Vec::new();
        let mut missing_gates = Vec::new();

        for item in &completed_plan_items {
            match matching_reality_gate(item, &reality_gates) {
                Some(row) if is_incomplete_reality_status(&row.status) => {
                    contradictions.push(RealityGateContradiction {
                        workstream: row.workstream.clone(),
                        reality_status: row.status.clone(),
                    });
                }
                Some(_) => {}
                None => missing_gates.push(MissingRealityGate {
                    workstream: item.clone(),
                }),
            }
        }

        Self {
            completed_plan_items,
            reality_gates,
            contradictions,
            missing_gates,
        }
    }

    pub fn is_in_sync(&self) -> bool {
        self.contradictions.is_empty() && self.missing_gates.is_empty()
    }

    pub fn format_contradictions(&self) -> String {
        if self.contradictions.is_empty() && self.missing_gates.is_empty() {
            return "plan/status reality gates are in sync".to_string();
        }

        let mut lines = self
            .contradictions
            .iter()
            .map(|contradiction| {
                format!(
                    "- {}: plan marks implemented but reality gate is {}",
                    contradiction.workstream, contradiction.reality_status
                )
            })
            .collect::<Vec<_>>();
        lines.extend(self.missing_gates.iter().map(|missing| {
            format!(
                "- {}: plan marks implemented but no matching reality gate row exists",
                missing.workstream
            )
        }));
        lines.join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingDocumentTotal {
    pub path: String,
}

impl std::fmt::Display for MissingDocumentTotal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "missing cargo test total declaration in {}",
            self.path
        )
    }
}

impl std::error::Error for MissingDocumentTotal {}

fn parse_cargo_test_total(contents: &str) -> Option<usize> {
    contents.lines().find_map(|line| {
        let trimmed = line.trim();
        if !is_total_line(trimmed) {
            return None;
        }

        declared_test_total(trimmed)
    })
}

fn parse_completed_plan_items(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("- [x]") {
                return None;
            }
            let label = if let Some(start) = trimmed.find("**") {
                let rest = &trimmed[(start + 2)..];
                rest.find("**").map(|end| &rest[..end]).unwrap_or(rest)
            } else {
                trimmed.trim_start_matches("- [x]").trim()
            };
            Some(strip_priority_prefix(label).to_string())
        })
        .collect()
}

fn parse_reality_gate_rows(contents: &str) -> Vec<RealityGateDocumentRow> {
    contents
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') || trimmed.contains("| ---") {
                return None;
            }
            let cells = trimmed
                .trim_matches('|')
                .split('|')
                .map(|cell| clean_markdown_cell(cell.trim()))
                .collect::<Vec<_>>();
            if cells.len() < 3 {
                return None;
            }
            let status = cells[2].clone();
            if !is_known_reality_status(&status) {
                return None;
            }
            Some(RealityGateDocumentRow {
                workstream: cells[1].clone(),
                status,
            })
        })
        .collect()
}

fn matching_reality_gate<'a>(
    item: &str,
    rows: &'a [RealityGateDocumentRow],
) -> Option<&'a RealityGateDocumentRow> {
    rows.iter().find(|row| names_match(item, &row.workstream))
}

fn names_match(plan_item: &str, workstream: &str) -> bool {
    let plan = normalized_name(plan_item);
    let gate = normalized_name(workstream);
    plan == gate || plan.contains(&gate) || gate.contains(&plan)
}

fn normalized_name(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .replace("v2", "")
        .replace("gate checker", "")
        .replace("gate", "")
        .replace("artifact", "")
        .replace("template", "")
        .replace("first runtime slice", "")
        .replace("first slice", "")
        .replace("read-only inventory", "")
        .replace("contracts", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_priority_prefix(value: &str) -> &str {
    value
        .split_once(' ')
        .filter(|(prefix, _)| {
            prefix.len() == 4
                && prefix.starts_with('P')
                && prefix.as_bytes()[1].is_ascii_digit()
                && prefix.as_bytes()[2] == b'.'
                && prefix.as_bytes()[3].is_ascii_digit()
        })
        .map(|(_, rest)| rest.trim())
        .unwrap_or(value.trim())
}

fn clean_markdown_cell(cell: &str) -> String {
    cell.trim_matches('`').trim().to_string()
}

fn is_known_reality_status(status: &str) -> bool {
    matches!(
        status,
        "implemented"
            | "implemented-unmerged"
            | "partial"
            | "simulation-only"
            | "planning-gate"
            | "not-implemented"
    )
}

fn is_incomplete_reality_status(status: &str) -> bool {
    matches!(
        status,
        "implemented-unmerged"
            | "partial"
            | "simulation-only"
            | "planning-gate"
            | "not-implemented"
    )
}

fn is_total_line(line: &str) -> bool {
    line.contains("cargo test")
        || line.contains("合计")
        || line.contains("全量通过")
        || line.contains("全量测试")
}

fn declared_test_total(line: &str) -> Option<usize> {
    let chars = line.char_indices().collect::<Vec<_>>();
    for (position, (start, ch)) in chars.iter().enumerate() {
        if !ch.is_ascii_digit() {
            continue;
        }
        if position > 0 && chars[position - 1].1.is_ascii_digit() {
            continue;
        }

        let end = line[*start..]
            .char_indices()
            .find_map(|(offset, ch)| (!ch.is_ascii_digit()).then_some(*start + offset))
            .unwrap_or(line.len());
        let suffix = line[end..]
            .trim_start_matches(|ch: char| ch == '`' || ch.is_whitespace())
            .trim_start();
        if suffix.starts_with("个测试") || suffix.starts_with("tests") {
            return line[*start..end].parse().ok();
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_test_list_counts_only_test_lines() {
        let output = "\
running tests/foo.rs
foo: test
bar: test
doc-test agent_llm_mm
";

        assert_eq!(CargoTestList::parse(output).total, 2);
    }

    #[test]
    fn document_total_parses_cargo_test_summary_lines() {
        let contents = "- `cargo test` 全量通过，共 270 个测试";

        assert_eq!(
            DocumentTestTotal::parse("README.md", contents),
            Some(DocumentTestTotal {
                path: "README.md".to_string(),
                total: 270,
            })
        );
    }

    #[test]
    fn document_total_ignores_dates_before_cargo_test_total() {
        let contents = "截至 `2026-05-19`，`cargo test` 全量通过，共 270 个测试";

        assert_eq!(
            DocumentTestTotal::parse("docs/testing-guide.md", contents)
                .expect("total should parse")
                .total,
            270
        );
    }

    #[test]
    fn document_total_accepts_markdown_wrapped_total_number() {
        let contents = "当前分支 `cargo test` 全量通过 `264` 个测试";

        assert_eq!(
            DocumentTestTotal::parse("docs/progress-tracker.md", contents)
                .expect("total should parse")
                .total,
            264
        );
    }

    #[test]
    fn document_total_ignores_priority_numbers_without_test_total_suffix() {
        let contents = "| `P0` | Local Alpha evidence summary | `cargo test --test local_alpha_release_evidence -v` |";

        assert!(
            DocumentTestTotal::parse("docs/product/follow-up-reality-gates.md", contents).is_none()
        );
    }

    #[test]
    fn report_lists_drifted_documents() {
        let report = StatusSyncReport::from_totals(
            256,
            vec![DocumentTestTotal {
                path: "README.md".to_string(),
                total: 255,
            }],
        );

        assert_eq!(
            report.format_mismatches(),
            "- README.md: documented 255, actual 256"
        );
    }

    #[test]
    fn report_from_document_contents_requires_every_document_to_declare_total() {
        let error = report_from_document_contents(255, [("docs/missing.md", "no total here")])
            .expect_err("missing totals should be reported");

        assert_eq!(error.path, "docs/missing.md");
    }
}
