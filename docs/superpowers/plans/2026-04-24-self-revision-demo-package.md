# Self-Revision Demo Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 增加一套零外网依赖、可重复重放的 demo package，证明 baseline memory、显式 conflict 触发的 automatic self-revision、durable commitment 更新，以及 snapshot before / after 下的 decision 变化。

**Architecture:** 新增两个辅助 binary：一个本地 `openai-compatible` stub HTTP 服务，按现有 `ModelPort` 协议稳定返回 self-revision proposal 和 decision action；一个 demo runner，复用现有 MCP `stdio` 服务和现有 4 个 tool 完整跑 canonical scenario，查询 SQLite durable state，并把 JSON/Markdown 产物写到 `target/reports/self-revision-demo/<timestamp>/`。运行时逻辑不扩展新的 MCP tool，不改 automatic self-revision 的 hook 语义，只把现有能力整理成可证明的证据链。实现默认保持 Rust-only，不把 `python3`、`jq` 或 `sqlite3` 作为新的运行前提。

**Tech Stack:** Rust 2024, Tokio, Reqwest, RMCP stdio, SQLite + SQLx, Serde JSON, shell wrapper, Markdown + Mermaid

---

## File Structure

- Create: `src/bin/demo_openai_compatible_stub.rs`
  - 本地 `/chat/completions` stub server，按请求提示词区分 decision / self-revision
- Create: `src/bin/run_self_revision_demo.rs`
  - orchestration runner，启动 stub、启动 MCP stdio server、调用 tool、查询 SQLite、写 artifacts / report
- Create: `tests/demo_openai_compatible_stub.rs`
  - 锁定 stub 的 HTTP contract 与 deterministic response
- Create: `tests/self_revision_demo_runner.rs`
  - 锁定 canonical scenario、artifact 输出和 shell wrapper smoke test
- Create: `scripts/run-self-revision-demo.sh`
  - macOS 一键入口，先 build binaries，再执行 runner
- Create: `examples/agent-llm-mm.demo.example.toml`
  - demo-only 配置模板，指向本地 stub provider
- Create: `docs/self-revision-demo-guide-2026-04-24.md`
  - 运行说明、artifact 解释、手工复演方法
- Create: `docs/reports/self-revision-demo-2026-04-24.md`
  - canonical report，由 runner 产物刷新
- Modify: `README.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/development-macos.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/document-map.md`

---

### Task 1: Build The Deterministic Local Stub Provider

**Files:**
- Create: `src/bin/demo_openai_compatible_stub.rs`
- Test: `tests/demo_openai_compatible_stub.rs`

- [x] **Step 1: Write the failing integration test for the stub contract**

```rust
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn demo_stub_distinguishes_decision_and_self_revision_requests() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_demo_openai_compatible_stub"))
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn demo stub");

    let stdout = child.stdout.take().expect("stub stdout");
    let mut startup_line = String::new();
    BufReader::new(stdout)
        .read_line(&mut startup_line)
        .expect("startup line");
    let startup: Value = serde_json::from_str(startup_line.trim()).expect("startup json");
    let base_url = startup["base_url"].as_str().expect("base_url");

    let client = reqwest::Client::new();

    let decision_response = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth("demo-test-key")
        .json(&json!({
            "model": "demo-local",
            "temperature": 0.0,
            "messages": [
                {
                    "role": "system",
                    "content": "Return only the next action name as plain text with no explanation."
                },
                {
                    "role": "user",
                    "content": "Task: review\nAction: review_conflicting_commitment_update\nSnapshot:\n{\n  \"identity\": [\"identity:self=architect\"],\n  \"commitments\": [],\n  \"claims\": [],\n  \"evidence\": [],\n  \"episodes\": []\n}"
                }
            ]
        }))
        .send()
        .await
        .expect("decision response")
        .json::<Value>()
        .await
        .expect("decision json");

    assert_eq!(
        decision_response["choices"][0]["message"]["content"],
        json!("apply_commitment_update_now")
    );

    let self_revision_response = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth("demo-test-key")
        .json(&json!({
            "model": "demo-local",
            "temperature": 0.0,
            "messages": [
                {
                    "role": "system",
                    "content": "Return only a JSON self-revision proposal with should_reflect, rationale, machine_patch.identity_patch, machine_patch.commitment_patch, proposed_evidence_event_ids, proposed_evidence_query, and confidence."
                },
                {
                    "role": "user",
                    "content": "Self revision request:\n{\n  \"trigger_type\": \"Conflict\",\n  \"namespace\": \"self\",\n  \"snapshot\": {\n    \"identity\": [\"identity:self=architect\"],\n    \"commitments\": [\"forbid:write_identity_core_directly\"],\n    \"claims\": [\"self:self.role is architect\"],\n    \"evidence\": [\"event:evt-1\"],\n    \"episodes\": [\"episode:demo-baseline\"]\n  },\n  \"evidence_event_ids\": [\"evt-1\"],\n  \"trigger_hints\": [\"conflict\", \"commitment\"]\n}"
                }
            ]
        }))
        .send()
        .await
        .expect("proposal response")
        .json::<Value>()
        .await
        .expect("proposal json");

    let proposal = self_revision_response["choices"][0]["message"]["content"]
        .as_str()
        .expect("proposal content");

    assert!(proposal.contains("\"should_reflect\":true"));
    assert!(proposal.contains("prefer:confirm_conflicting_commitment_updates_before_overwrite"));

    let _ = child.kill();
}
```

- [x] **Step 2: Run the test to verify it fails**

Run: `cargo test --test demo_openai_compatible_stub demo_stub_distinguishes_decision_and_self_revision_requests -v`

Expected: FAIL with a missing `demo_openai_compatible_stub` binary target

- [x] **Step 3: Implement the stub binary**

```rust
use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
    let port = parse_port(std::env::args().skip(1))?;
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    let base_url = format!("http://{}", listener.local_addr()?);

    println!("{}", json!({ "base_url": base_url }));

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = handle_connection(&mut stream).await {
                eprintln!("demo stub request failed: {error}");
            }
        });
    }
}

fn parse_port(mut args: impl Iterator<Item = String>) -> Result<u16> {
    match (args.next().as_deref(), args.next()) {
        (Some("--port"), Some(value)) => Ok(value.parse::<u16>()?),
        _ => Ok(0),
    }
}

async fn handle_connection(stream: &mut tokio::net::TcpStream) -> Result<()> {
    let mut buffer = vec![0_u8; 64 * 1024];
    let bytes_read = stream.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let body = request.split("\r\n\r\n").nth(1).context("missing http body")?;
    let json_body: Value = serde_json::from_str(body)?;

    let content = classify_response(&json_body)?;
    let response = json!({
        "id": "chatcmpl-demo",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            }
        }]
    });

    let response_text = response.to_string();
    let http = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        response_text.len(),
        response_text
    );
    stream.write_all(http.as_bytes()).await?;
    Ok(())
}

fn classify_response(request: &Value) -> Result<String> {
    let system = request["messages"][0]["content"].as_str().unwrap_or_default();
    let user = request["messages"][1]["content"].as_str().unwrap_or_default();

    if system.contains("Return only the next action name") && user.contains("Task:") {
        let action = if user.contains("prefer:confirm_conflicting_commitment_updates_before_overwrite") {
            "confirm_conflicting_commitment_updates_before_overwrite"
        } else {
            "apply_commitment_update_now"
        };
        return Ok(action.to_string());
    }

    if system.contains("Return only a JSON self-revision proposal") && user.contains("Self revision request:") {
        return Ok(json!({
            "should_reflect": true,
            "rationale": "Conflict evidence suggests tighter commitment hygiene.",
            "machine_patch": {
                "identity_patch": null,
                "commitment_patch": {
                    "commitments": [
                        "prefer:confirm_conflicting_commitment_updates_before_overwrite"
                    ]
                }
            }
        })
        .to_string());
    }

    Ok("unsupported_demo_request".to_string())
}
```

- [x] **Step 4: Run the targeted tests to verify the stub passes**

Run: `cargo test --test demo_openai_compatible_stub --test openai_compatible_model -v`

Expected: PASS with the new stub contract test and no regressions in `openai_compatible_model`

- [ ] **Step 5: Commit**

```bash
git add src/bin/demo_openai_compatible_stub.rs tests/demo_openai_compatible_stub.rs
git commit -m "feat: add deterministic demo stub provider"
```

### Task 2: Implement The Demo Runner And Lock The Scenario With An Integration Test

**Files:**
- Create: `src/bin/run_self_revision_demo.rs`
- Create: `tests/self_revision_demo_runner.rs`

- [x] **Step 1: Write the failing end-to-end runner test**

```rust
use std::{fs, process::Command};

use serde_json::Value;

#[test]
fn demo_runner_writes_expected_artifacts_and_proves_decision_shift() {
    let output_dir = tempfile::tempdir().expect("tempdir");

    let status = Command::new(env!("CARGO_BIN_EXE_run_self_revision_demo"))
        .arg("--output-dir")
        .arg(output_dir.path())
        .status()
        .expect("run demo runner");

    assert!(status.success());

    for name in [
        "doctor.json",
        "snapshot-before.json",
        "snapshot-after.json",
        "decision-before.json",
        "decision-after.json",
        "timeline.json",
        "sqlite-summary.json",
        "report.md",
    ] {
        assert!(
            output_dir.path().join(name).exists(),
            "missing demo artifact: {name}"
        );
    }

    let doctor: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("doctor.json")).expect("doctor"),
    )
    .expect("doctor json");
    assert_eq!(doctor["self_revision_write_path"], "run_reflection");

    let snapshot_before: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("snapshot-before.json")).expect("snapshot before"),
    )
    .expect("snapshot before json");
    let snapshot_after: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("snapshot-after.json")).expect("snapshot after"),
    )
    .expect("snapshot after json");

    assert!(
        snapshot_before["commitments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "forbid:write_identity_core_directly")
    );
    assert!(
        snapshot_after["commitments"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "prefer:confirm_conflicting_commitment_updates_before_overwrite")
    );

    let decision_before: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("decision-before.json")).expect("decision before"),
    )
    .expect("decision before json");
    let decision_after: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("decision-after.json")).expect("decision after"),
    )
    .expect("decision after json");

    assert_eq!(decision_before["blocked"], false);
    assert_eq!(decision_after["blocked"], false);
    assert_eq!(decision_before["decision"]["action"], "apply_commitment_update_now");
    assert_eq!(
        decision_after["decision"]["action"],
        "confirm_conflicting_commitment_updates_before_overwrite"
    );

    let sqlite_summary: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("sqlite-summary.json")).expect("sqlite summary"),
    )
    .expect("sqlite summary json");
    assert_eq!(
        sqlite_summary["reflection_trigger_ledger"][0]["status"],
        "handled"
    );
    assert_eq!(
        sqlite_summary["reflection_trigger_ledger"][0]["trigger_type"],
        "conflict"
    );

    let timeline: Value = serde_json::from_slice(
        &fs::read(output_dir.path().join("timeline.json")).expect("timeline"),
    )
    .expect("timeline json");
    assert_eq!(timeline["gate_before"]["blocked"], true);
    assert_eq!(timeline["negative_conflict"]["handled_conflict_rows"], 0);
    assert_eq!(timeline["positive_conflict"]["handled_conflict_rows"], 1);
}
```

- [x] **Step 2: Run the test to verify it fails**

Run: `cargo test --test self_revision_demo_runner demo_runner_writes_expected_artifacts_and_proves_decision_shift -v`

Expected: FAIL with a missing `run_self_revision_demo` binary target

- [x] **Step 3: Implement the runner binary**

```rust
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use agent_llm_mm::{run_doctor, support::config::AppConfig};
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use sqlx::{Row, sqlite::SqlitePool};

#[derive(Debug, Serialize)]
struct SqliteSummary {
    commitments: Vec<String>,
    reflection_trigger_ledger: Vec<Value>,
    reflections: Vec<Value>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let output_dir = parse_output_dir(std::env::args().skip(1))?;
    fs::create_dir_all(&output_dir)?;

    let stub_bin = sibling_binary("demo_openai_compatible_stub")?;
    let server_bin = sibling_binary("agent_llm_mm")?;
    let database_path = output_dir.join("demo.sqlite");
    let database_url = format!("sqlite://{}", database_path.display());
    let config_path = output_dir.join("agent-llm-mm.demo.toml");

    let mut stub = Command::new(stub_bin)
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()?;
    let stub_base_url = read_stub_base_url(&mut stub)?;
    fs::write(
        &config_path,
        format!(
            "transport = \"stdio\"\ndatabase_url = \"{database_url}\"\n\n[model]\nprovider = \"openai-compatible\"\n\n[model.openai_compatible]\nbase_url = \"{stub_base_url}\"\napi_key = \"demo-local-key\"\nmodel = \"demo-local\"\ntimeout_ms = 30000\n"
        ),
    )?;

    let config = AppConfig::load_from_path(&config_path).map_err(anyhow::Error::msg)?;
    let doctor = run_doctor(config.clone()).await?;
    write_json(&output_dir.join("doctor.json"), &doctor)?;

    let mut client = StdioClient::spawn(&server_bin, &config_path)?;
    let _ = client.list_all_tools()?;

    let baseline = client.call_tool("ingest_interaction", json!({
        "event": { "owner": "User", "kind": "Conversation", "summary": "baseline memory event" },
        "claim_drafts": [{
            "owner": "Self_",
            "subject": "self.role",
            "predicate": "is",
            "object": "architect",
            "mode": "Observed"
        }],
        "episode_reference": "episode:demo-baseline"
    }))?;

    let snapshot_before = client.call_tool("build_self_snapshot", json!({ "budget": 8 }))?;
    let snapshot_before_value = snapshot_before["result"]["structuredContent"]["snapshot"].clone();
    write_json(&output_dir.join("snapshot-before.json"), &snapshot_before_value)?;

    let gate_before = client.call_tool("decide_with_snapshot", json!({
        "task": "attempt forbidden direct identity write",
        "action": "write_identity_core_directly",
        "snapshot": snapshot_before_value
    }))?;

    let _ = client.call_tool("ingest_interaction", json!({
        "event": {
            "owner": "Self_",
            "kind": "Action",
            "summary": "self attempted a conflicting commitment overwrite"
        },
        "claim_drafts": [],
        "episode_reference": "episode:demo-conflict-negative"
    }))?;
    let negative_handled_conflict_rows = count_handled_conflicts(&database_url).await?;

    let _ = client.call_tool("ingest_interaction", json!({
        "event": {
            "owner": "Self_",
            "kind": "Action",
            "summary": "self attempted a commitment overwrite that requires confirmation"
        },
        "claim_drafts": [],
        "episode_reference": "episode:demo-conflict-positive",
        "trigger_hints": ["conflict", "commitment"]
    }))?;
    let positive_handled_conflict_rows = count_handled_conflicts(&database_url).await?;

    let snapshot_after = client.call_tool("build_self_snapshot", json!({ "budget": 8 }))?;
    let snapshot_after_value = snapshot_after["result"]["structuredContent"]["snapshot"].clone();
    write_json(&output_dir.join("snapshot-after.json"), &snapshot_after_value)?;

    let decision_before = client.call_tool("decide_with_snapshot", json!({
        "task": "review update",
        "action": "review_conflicting_commitment_update",
        "snapshot": snapshot_before_value
    }))?;
    write_json(
        &output_dir.join("decision-before.json"),
        &decision_before["result"]["structuredContent"],
    )?;

    let decision_after = client.call_tool("decide_with_snapshot", json!({
        "task": "review update",
        "action": "review_conflicting_commitment_update",
        "snapshot": snapshot_after_value
    }))?;
    write_json(
        &output_dir.join("decision-after.json"),
        &decision_after["result"]["structuredContent"],
    )?;

    let sqlite_summary = query_sqlite_summary(&database_url)?;
    write_json(&output_dir.join("sqlite-summary.json"), &sqlite_summary)?;
    write_json(
        &output_dir.join("timeline.json"),
        &json!({
            "baseline": {
                "event_id": baseline["result"]["structuredContent"]["event_id"]
            },
            "gate_before": gate_before["result"]["structuredContent"],
            "negative_conflict": {
                "handled_conflict_rows": negative_handled_conflict_rows
            },
            "positive_conflict": {
                "handled_conflict_rows": positive_handled_conflict_rows
            }
        }),
    )?;
    fs::write(output_dir.join("report.md"), render_report(&doctor, &sqlite_summary)?)?;

    Ok(())
}

async fn count_handled_conflicts(database_url: &str) -> anyhow::Result<i64> {
    let pool = SqlitePool::connect(database_url).await?;
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM reflection_trigger_ledger WHERE trigger_type = 'conflict' AND status = 'handled'",
    )
    .fetch_one(&pool)
    .await?;
    Ok(count)
}

fn sibling_binary(name: &str) -> anyhow::Result<PathBuf> {
    let mut dir = std::env::current_exe()?;
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    Ok(dir.join(name))
}
```

- [x] **Step 4: Run the targeted runner tests to verify they pass**

Run: `cargo test --test self_revision_demo_runner -v`

Expected: PASS with artifact existence, commitment diff, handled ledger, and decision-shift assertions

- [ ] **Step 5: Commit**

```bash
git add src/bin/run_self_revision_demo.rs tests/self_revision_demo_runner.rs
git commit -m "feat: add self-revision demo runner"
```

### Task 3: Add The macOS Wrapper Script And The Demo Config Example

**Files:**
- Create: `scripts/run-self-revision-demo.sh`
- Create: `examples/agent-llm-mm.demo.example.toml`
- Modify: `tests/self_revision_demo_runner.rs`

- [x] **Step 1: Extend the runner test with a shell-wrapper smoke test**

```rust
#[test]
fn shell_wrapper_runs_demo_runner_and_writes_report() {
    let output_dir = tempfile::tempdir().expect("tempdir");

    let output = Command::new("bash")
        .arg("scripts/run-self-revision-demo.sh")
        .arg(output_dir.path())
        .output()
        .expect("run shell wrapper");

    assert!(output.status.success(), "{output:?}");
    assert!(output_dir.path().join("report.md").exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("self-revision demo artifacts"));
}
```

- [x] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test --test self_revision_demo_runner shell_wrapper_runs_demo_runner_and_writes_report -v`

Expected: FAIL because `scripts/run-self-revision-demo.sh` does not exist yet

- [x] **Step 3: Add the shell wrapper and the demo config example**

```bash
#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "${script_dir}/.." && pwd)"
requested_output_dir="${1:-${project_root}/target/reports/self-revision-demo/$(date +%Y%m%d-%H%M%S)}"

cd "${project_root}"

cargo build --bins
"${project_root}/target/debug/run_self_revision_demo" --output-dir "${requested_output_dir}"

printf 'self-revision demo artifacts: %s\n' "${requested_output_dir}"
```

```toml
transport = "stdio"
database_url = "sqlite:///absolute/path/to/demo.sqlite"

[model]
provider = "openai-compatible"

[model.openai_compatible]
base_url = "http://127.0.0.1:8787"
api_key = "demo-local-key"
model = "demo-local"
timeout_ms = 30000
```

- [x] **Step 4: Re-run tests and perform one manual wrapper dry run**

Run: `cargo test --test self_revision_demo_runner -v`

Expected: PASS with the shell-wrapper smoke test included

Run: `bash scripts/run-self-revision-demo.sh "$(mktemp -d)"`

Expected: PASS and stdout contains `self-revision demo artifacts:`

- [ ] **Step 5: Commit**

```bash
git add scripts/run-self-revision-demo.sh examples/agent-llm-mm.demo.example.toml tests/self_revision_demo_runner.rs
git commit -m "feat: add self-revision demo shell entry"
```

### Task 4: Write The Guide, Refresh The Canonical Report, And Update Project Docs

**Files:**
- Create: `docs/self-revision-demo-guide-2026-04-24.md`
- Create: `docs/reports/self-revision-demo-2026-04-24.md`
- Modify: `README.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `docs/development-macos.md`
- Modify: `docs/project-status.md`
- Modify: `docs/roadmap.md`
- Modify: `docs/document-map.md`

- [x] **Step 1: Write the demo guide**

```md
# Self-Revision Demo Guide

## Goal

用一条本地、零外网依赖的 canonical scenario 证明：

- baseline memory 已真实落盘
- 无 explicit conflict hints 时不会误触发 auto-reflection
- 有 explicit conflict hints 时会产生 handled trigger ledger + reflection audit
- updated commitments 会进入后续 snapshot
- 同一个 allowed action 在 before / after snapshot 下会得到不同 decision

## Run

```zsh
./scripts/run-self-revision-demo.sh
```

## Artifacts

- `doctor.json`
- `snapshot-before.json`
- `snapshot-after.json`
- `decision-before.json`
- `decision-after.json`
- `timeline.json`
- `sqlite-summary.json`
- `report.md`
```

- [x] **Step 2: Refresh the canonical report from a real runner execution**

```md
# Self-Revision Demo Report

## Environment

- transport: `stdio`
- durable write path: `run_reflection`
- runtime hooks:
  - `ingest_interaction:failure`
  - `ingest_interaction:conflict`
  - `decide_with_snapshot:conflict`
  - `build_self_snapshot:periodic`

## Snapshot Diff

| Phase | Commitments |
| --- | --- |
| Before | `forbid:write_identity_core_directly` |
| After | `forbid:write_identity_core_directly`, `prefer:confirm_conflicting_commitment_updates_before_overwrite` |

## Decision Shift

| Phase | Blocked | Decision |
| --- | --- | --- |
| Before | `false` | `apply_commitment_update_now` |
| After | `false` | `confirm_conflicting_commitment_updates_before_overwrite` |
```

- [x] **Step 3: Update README and project docs with the new demo entry**

```md
## 演示入口

如果你想快速验证当前 automatic self-revision MVP 的真实效果，可以直接运行：

```zsh
./scripts/run-self-revision-demo.sh
```

该命令会启动本地 demo stub provider、跑完整 canonical scenario，并把 artifact 输出到 `target/reports/self-revision-demo/...`。
```

```md
## 额外验证：self-revision demo package

```zsh
./scripts/run-self-revision-demo.sh
```

预期输出：

- `doctor.json` 显示当前 runtime hooks 和 durable write path
- `sqlite-summary.json` 包含 handled conflict ledger
- `report.md` 显示 snapshot diff 和 decision before / after
```

- [x] **Step 4: Run formatting, linting, targeted tests, and the demo runner**

Run: `cargo fmt --check`

Expected: PASS

Run: `cargo clippy --all-targets --all-features -- -D warnings`

Expected: PASS

Run: `cargo test --test demo_openai_compatible_stub --test self_revision_demo_runner --test openai_compatible_model --test mcp_stdio -v`

Expected: PASS with the new stub and runner coverage plus no regression in existing integration suites

Run: `./scripts/run-self-revision-demo.sh target/reports/self-revision-demo/latest`

Expected: PASS and all eight artifact files exist under `target/reports/self-revision-demo/latest`

- [ ] **Step 5: Commit**

```bash
git add README.md docs/testing-guide-2026-03-24.md docs/development-macos.md docs/project-status.md docs/roadmap.md docs/document-map.md docs/self-revision-demo-guide-2026-04-24.md docs/reports/self-revision-demo-2026-04-24.md
git commit -m "docs: add self-revision demo package docs"
```
