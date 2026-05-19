pub const TEST_TOTAL_DOCUMENTS: &[&str] = &[
    "README.md",
    "docs/testing-guide-2026-03-24.md",
    "docs/local-mcp-integration-2026-03-26.md",
    "docs/project-status.md",
    "docs/progress-tracker.md",
    "docs/product/follow-up-reality-gates.md",
];

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
