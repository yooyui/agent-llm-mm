# Dashboard Production Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a configurable production dashboard service that starts with `agent_llm_mm serve`, records read-only runtime operation events, and serves the approved cute anime dashboard UI without corrupting MCP `stdio`.

**Architecture:** Add a focused `src/interfaces/dashboard/` module with function-oriented boundaries: pure normalization/projection helpers, a bounded recorder, embedded assets, and an `axum` HTTP boundary. `serve` starts the dashboard only when configured, injects a no-op or enabled observer into the existing MCP runtime, and keeps all dashboard logging off MCP `stdout`.

**Tech Stack:** Rust 2024, `tokio`, `axum`, `serde`, `serde_json`, `chrono`, existing MCP `stdio` server, existing SQLite and model adapters.

---

## Baseline

Worktree:

```text
/tmp/agent-llm-mm-dashboard-production-service
```

Baseline command already verified before this plan:

```zsh
cargo test
```

Expected baseline: 131 tests pass when run outside the sandbox because existing tests launch local HTTP stub services and the default doctor path writes the user-scoped SQLite database.

## File Map

- Modify `Cargo.toml`
  - Add direct HTTP/SSE dependencies.
- Modify `src/interfaces/mod.rs`
  - Export the dashboard module.
- Create `src/interfaces/dashboard/mod.rs`
  - Public dashboard API and no-op/enabled observer wrapper.
- Create `src/interfaces/dashboard/event.rs`
  - Operation event types and normalization functions.
- Create `src/interfaces/dashboard/recorder.rs`
  - Bounded event recorder and broadcast support for SSE.
- Create `src/interfaces/dashboard/projection.rs`
  - Summary, event filtering, and detail projection functions.
- Create `src/interfaces/dashboard/assets.rs`
  - Embedded HTML/CSS/JS for `Memory-chan Live Cockpit`.
- Create `src/interfaces/dashboard/http.rs`
  - `axum` routes and dashboard server lifecycle.
- Modify `src/support/config.rs`
  - Load `[dashboard]` from TOML and expose dashboard config through `AppConfig`.
- Modify `src/support/doctor.rs`
  - Report dashboard configuration without starting HTTP.
- Modify `src/interfaces/mcp/server.rs`
  - Start dashboard for `serve` and record MCP tool / auto-reflection events through the observer.
- Modify `examples/agent-llm-mm.example.toml`
  - Document production dashboard settings.
- Modify docs:
  - `README.md`
  - `docs/project-status.md`
  - `docs/development-macos.md`
  - `docs/testing-guide-2026-03-24.md`
- Add tests:
  - `tests/dashboard_config.rs`
  - `tests/dashboard_recorder.rs`
  - `tests/dashboard_projection.rs`
  - `tests/dashboard_http.rs`
  - Add targeted cases to `tests/bootstrap.rs` and `tests/mcp_stdio.rs`.

---

### Task 1: Dashboard Config And Doctor Report

**Files:**
- Modify: `src/support/config.rs`
- Modify: `src/support/doctor.rs`
- Create: `tests/dashboard_config.rs`
- Modify: `tests/bootstrap.rs`

- [ ] **Step 1: Write failing dashboard config tests**

Create `tests/dashboard_config.rs`:

```rust
use agent_llm_mm::support::config::AppConfig;

#[test]
fn default_dashboard_config_is_disabled_and_safe() {
    let config = AppConfig::default();

    assert!(!config.dashboard.enabled);
    assert_eq!(config.dashboard.host, "127.0.0.1");
    assert_eq!(config.dashboard.port, 8787);
    assert_eq!(config.dashboard.base_path, "/");
    assert_eq!(config.dashboard.event_capacity, 2000);
    assert!(config.dashboard.sse_enabled);
    assert!(!config.dashboard.open_browser);
    assert!(!config.dashboard.required);
    assert!(config.dashboard.validate().is_ok());
}

#[test]
fn load_from_path_reads_dashboard_section() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let config_path = temp_dir.path().join("agent-llm-mm.local.toml");
    std::fs::write(
        &config_path,
        r#"
transport = "stdio"
database_url = "sqlite:///tmp/agent-llm-mm-dashboard-config-test.sqlite"

[dashboard]
enabled = true
host = "0.0.0.0"
port = 9797
base_path = "/agent-llm-mm"
event_capacity = 123
sse_enabled = false
open_browser = true
required = true
"#,
    )
    .expect("write config");

    let config = AppConfig::load_from_path(&config_path).expect("load config");

    assert!(config.dashboard.enabled);
    assert_eq!(config.dashboard.host, "0.0.0.0");
    assert_eq!(config.dashboard.port, 9797);
    assert_eq!(config.dashboard.base_path, "/agent-llm-mm");
    assert_eq!(config.dashboard.event_capacity, 123);
    assert!(!config.dashboard.sse_enabled);
    assert!(config.dashboard.open_browser);
    assert!(config.dashboard.required);
    assert!(config.dashboard.validate().is_ok());
}

#[test]
fn dashboard_rejects_zero_event_capacity() {
    let mut config = AppConfig::default();
    config.dashboard.event_capacity = 0;

    assert_eq!(
        config.dashboard.validate().unwrap_err(),
        "dashboard.event_capacity must be greater than 0"
    );
}

#[test]
fn dashboard_rejects_base_path_without_leading_slash() {
    let mut config = AppConfig::default();
    config.dashboard.base_path = "agent-llm-mm".to_string();

    assert_eq!(
        config.dashboard.validate().unwrap_err(),
        "dashboard.base_path must start with /"
    );
}
```

- [ ] **Step 2: Write failing doctor report test**

Append this test to `tests/bootstrap.rs`:

```rust
#[tokio::test]
async fn doctor_reports_dashboard_config_without_starting_dashboard() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let database_url = format!(
        "sqlite://{}",
        temp_dir
            .path()
            .join("doctor-dashboard.sqlite")
            .to_string_lossy()
            .replace('\\', "/")
    );
    let config = AppConfig {
        transport: TransportKind::Stdio,
        database_url,
        model_provider: ModelProviderKind::Mock,
        model_config: ModelConfig::Mock,
        dashboard: agent_llm_mm::support::config::DashboardConfig {
            enabled: true,
            host: "127.0.0.1".to_string(),
            port: 8787,
            base_path: "/agent-llm-mm".to_string(),
            event_capacity: 2000,
            sse_enabled: true,
            open_browser: false,
            required: true,
        },
    };

    let report = run_doctor(config).await.expect("doctor should pass");

    assert!(report.dashboard_enabled);
    assert_eq!(report.dashboard_host, "127.0.0.1");
    assert_eq!(report.dashboard_port, 8787);
    assert_eq!(report.dashboard_base_path, "/agent-llm-mm");
    assert!(report.dashboard_required);
}
```

- [ ] **Step 3: Run tests and verify they fail**

Run:

```zsh
cargo test --test dashboard_config --test bootstrap doctor_reports_dashboard_config_without_starting_dashboard -v
```

Expected: FAIL because `AppConfig` has no `dashboard` field and `DashboardConfig` does not exist.

- [ ] **Step 4: Implement config type and TOML loading**

In `src/support/config.rs`, add the config type near `AppConfig`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub base_path: String,
    pub event_capacity: usize,
    pub sse_enabled: bool,
    pub open_browser: bool,
    pub required: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 8787,
            base_path: "/".to_string(),
            event_capacity: 2000,
            sse_enabled: true,
            open_browser: false,
            required: false,
        }
    }
}

impl DashboardConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.event_capacity == 0 {
            return Err("dashboard.event_capacity must be greater than 0".to_string());
        }
        if !self.base_path.starts_with('/') {
            return Err("dashboard.base_path must start with /".to_string());
        }
        Ok(())
    }
}
```

Add `pub dashboard: DashboardConfig` to `AppConfig`, initialize it in `Default`, load it in `load_from_path`, and add `AppConfig::validate()`:

```rust
pub fn validate(&self) -> Result<(), String> {
    self.validate_model_config()?;
    self.dashboard.validate()?;
    Ok(())
}
```

Update `run_doctor` and `Runtime::bootstrap` to call `config.validate()` instead of only `validate_model_config()`.

Add file config structs:

```rust
#[derive(Debug, Deserialize, Default)]
struct FileDashboardConfig {
    enabled: Option<bool>,
    host: Option<String>,
    port: Option<u16>,
    base_path: Option<String>,
    event_capacity: Option<usize>,
    sse_enabled: Option<bool>,
    open_browser: Option<bool>,
    required: Option<bool>,
}
```

Add `dashboard: Option<FileDashboardConfig>` to `FileConfig` and merge field-by-field into the default dashboard config.

- [ ] **Step 5: Implement doctor report fields**

In `src/support/doctor.rs`, extend `DoctorReport`:

```rust
pub dashboard_enabled: bool,
pub dashboard_host: String,
pub dashboard_port: u16,
pub dashboard_base_path: String,
pub dashboard_required: bool,
```

Call dashboard validation in `run_doctor` before runtime bootstrap:

```rust
config.dashboard.validate().map_err(anyhow::Error::msg)?;
```

Populate the new report fields from `config.dashboard`.

- [ ] **Step 6: Run tests and verify they pass**

Run:

```zsh
cargo test --test dashboard_config --test bootstrap doctor_reports_dashboard_config_without_starting_dashboard -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

```zsh
git add src/support/config.rs src/support/doctor.rs tests/dashboard_config.rs tests/bootstrap.rs
git commit -m "feat: add dashboard config"
```

---

### Task 2: Operation Events And Recorder

**Files:**
- Modify: `src/interfaces/mod.rs`
- Create: `src/interfaces/dashboard/mod.rs`
- Create: `src/interfaces/dashboard/event.rs`
- Create: `src/interfaces/dashboard/recorder.rs`
- Create: `tests/dashboard_recorder.rs`

- [ ] **Step 1: Write failing recorder tests**

Create `tests/dashboard_recorder.rs`:

```rust
use agent_llm_mm::interfaces::dashboard::{
    EventQuery, OperationEvent, OperationKind, OperationRecorder, OperationStatus,
};
use chrono::Utc;
use serde_json::json;

fn event(sequence: u64, kind: OperationKind, status: OperationStatus, namespace: Option<&str>) -> OperationEvent {
    OperationEvent {
        id: format!("op_{sequence}"),
        sequence,
        timestamp: Utc::now(),
        kind,
        status,
        operation: "ingest_interaction".to_string(),
        namespace: namespace.map(str::to_string),
        summary: format!("event {sequence}"),
        correlation_id: None,
        payload: json!({ "sequence": sequence }),
    }
}

#[test]
fn recorder_keeps_sequence_order_and_drops_oldest_after_capacity() {
    let recorder = OperationRecorder::new(2);

    recorder.append(event(1, OperationKind::Tool, OperationStatus::Started, Some("self")));
    recorder.append(event(2, OperationKind::Tool, OperationStatus::Ok, Some("self")));
    recorder.append(event(3, OperationKind::Reflection, OperationStatus::Handled, Some("self")));

    let events = recorder.recent(EventQuery::default());

    assert_eq!(events.iter().map(|event| event.sequence).collect::<Vec<_>>(), vec![2, 3]);
}

#[test]
fn recorder_filters_by_kind_status_and_namespace() {
    let recorder = OperationRecorder::new(5);
    recorder.append(event(1, OperationKind::Tool, OperationStatus::Ok, Some("self")));
    recorder.append(event(2, OperationKind::Reflection, OperationStatus::Handled, Some("self")));
    recorder.append(event(3, OperationKind::Reflection, OperationStatus::Rejected, Some("project/demo")));

    let query = EventQuery {
        limit: Some(10),
        kind: Some(OperationKind::Reflection),
        status: Some(OperationStatus::Handled),
        namespace: Some("self".to_string()),
    };

    let events = recorder.recent(query);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].sequence, 2);
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```zsh
cargo test --test dashboard_recorder -v
```

Expected: FAIL because `interfaces::dashboard` does not exist.

- [ ] **Step 3: Add dashboard module exports**

In `src/interfaces/mod.rs`, add:

```rust
pub mod dashboard;
```

Create `src/interfaces/dashboard/mod.rs`:

```rust
pub mod event;
pub mod recorder;

pub use event::{EventQuery, OperationEvent, OperationKind, OperationStatus};
pub use recorder::OperationRecorder;
```

- [ ] **Step 4: Implement event types**

Create `src/interfaces/dashboard/event.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Startup,
    Tool,
    Trigger,
    Reflection,
    Decision,
    Snapshot,
    Doctor,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Started,
    Ok,
    Handled,
    Suppressed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationEvent {
    pub id: String,
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub operation: String,
    pub namespace: Option<String>,
    pub summary: String,
    pub correlation_id: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventQuery {
    pub limit: Option<usize>,
    pub kind: Option<OperationKind>,
    pub status: Option<OperationStatus>,
    pub namespace: Option<String>,
}
```

- [ ] **Step 5: Implement bounded recorder**

Create `src/interfaces/dashboard/recorder.rs`:

```rust
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tokio::sync::broadcast;

use super::{EventQuery, OperationEvent};

#[derive(Debug, Clone)]
pub struct OperationRecorder {
    capacity: usize,
    events: Arc<Mutex<VecDeque<OperationEvent>>>,
    broadcaster: broadcast::Sender<OperationEvent>,
}

impl OperationRecorder {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "operation recorder capacity must be greater than 0");
        Self {
            capacity,
            events: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            broadcaster: broadcast::channel(capacity.max(16)).0,
        }
    }

    pub fn append(&self, event: OperationEvent) {
        let mut events = self.events.lock().expect("operation recorder lock poisoned");
        if events.len() == self.capacity {
            events.pop_front();
        }
        events.push_back(event.clone());
        let _ = self.broadcaster.send(event);
    }

    pub fn recent(&self, query: EventQuery) -> Vec<OperationEvent> {
        let limit = query.limit.unwrap_or(self.capacity);
        let events = self.events.lock().expect("operation recorder lock poisoned");
        events
            .iter()
            .filter(|event| query.kind.is_none_or(|kind| event.kind == kind))
            .filter(|event| query.status.is_none_or(|status| event.status == status))
            .filter(|event| {
                query
                    .namespace
                    .as_ref()
                    .is_none_or(|namespace| event.namespace.as_ref() == Some(namespace))
            })
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<OperationEvent> {
        self.broadcaster.subscribe()
    }
}
```

- [ ] **Step 6: Run tests**

Run:

```zsh
cargo test --test dashboard_recorder -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

```zsh
git add src/interfaces/mod.rs src/interfaces/dashboard/mod.rs src/interfaces/dashboard/event.rs src/interfaces/dashboard/recorder.rs tests/dashboard_recorder.rs
git commit -m "feat: add dashboard operation recorder"
```

---

### Task 3: Dashboard Projections

**Files:**
- Modify: `src/interfaces/dashboard/mod.rs`
- Create: `src/interfaces/dashboard/projection.rs`
- Create: `tests/dashboard_projection.rs`

- [ ] **Step 1: Write failing projection tests**

Create `tests/dashboard_projection.rs`:

```rust
use agent_llm_mm::interfaces::dashboard::{
    build_summary, project_event_detail, DashboardRuntimeInfo, OperationEvent, OperationKind,
    OperationStatus,
};
use chrono::Utc;
use serde_json::json;

fn event(sequence: u64, kind: OperationKind, status: OperationStatus) -> OperationEvent {
    OperationEvent {
        id: format!("op_{sequence}"),
        sequence,
        timestamp: Utc::now(),
        kind,
        status,
        operation: "run_reflection".to_string(),
        namespace: Some("self".to_string()),
        summary: "reflection handled".to_string(),
        correlation_id: Some("corr-1".to_string()),
        payload: json!({ "reflection_id": "reflection-1" }),
    }
}

#[test]
fn summary_counts_events_by_kind_and_status() {
    let events = vec![
        event(1, OperationKind::Tool, OperationStatus::Ok),
        event(2, OperationKind::Reflection, OperationStatus::Handled),
        event(3, OperationKind::Reflection, OperationStatus::Rejected),
    ];
    let runtime = DashboardRuntimeInfo {
        service_name: "agent-llm-mm".to_string(),
        transport: "stdio".to_string(),
        provider: "mock".to_string(),
        dashboard_enabled: true,
        read_only: true,
    };

    let summary = build_summary(&events, &runtime);

    assert_eq!(summary.total_events, 3);
    assert_eq!(summary.reflection_events, 2);
    assert_eq!(summary.failed_events, 0);
    assert_eq!(summary.runtime.service_name, "agent-llm-mm");
}

#[test]
fn detail_projection_preserves_payload_and_read_only_boundary() {
    let detail = project_event_detail(&event(7, OperationKind::Reflection, OperationStatus::Handled));

    assert_eq!(detail.id, "op_7");
    assert_eq!(detail.operation, "run_reflection");
    assert!(detail.read_only);
    assert_eq!(detail.payload["reflection_id"], "reflection-1");
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```zsh
cargo test --test dashboard_projection -v
```

Expected: FAIL because projection functions do not exist.

- [ ] **Step 3: Implement projection module**

Create `src/interfaces/dashboard/projection.rs`:

```rust
use serde::Serialize;
use serde_json::Value;

use super::{OperationEvent, OperationKind, OperationStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardRuntimeInfo {
    pub service_name: String,
    pub transport: String,
    pub provider: String,
    pub dashboard_enabled: bool,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DashboardSummary {
    pub runtime: DashboardRuntimeInfo,
    pub total_events: usize,
    pub tool_events: usize,
    pub reflection_events: usize,
    pub decision_events: usize,
    pub snapshot_events: usize,
    pub failed_events: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OperationDetail {
    pub id: String,
    pub operation: String,
    pub kind: OperationKind,
    pub status: OperationStatus,
    pub namespace: Option<String>,
    pub summary: String,
    pub payload: Value,
    pub read_only: bool,
}

pub fn build_summary(events: &[OperationEvent], runtime: &DashboardRuntimeInfo) -> DashboardSummary {
    DashboardSummary {
        runtime: runtime.clone(),
        total_events: events.len(),
        tool_events: count_kind(events, OperationKind::Tool),
        reflection_events: count_kind(events, OperationKind::Reflection),
        decision_events: count_kind(events, OperationKind::Decision),
        snapshot_events: count_kind(events, OperationKind::Snapshot),
        failed_events: events
            .iter()
            .filter(|event| event.status == OperationStatus::Failed)
            .count(),
    }
}

pub fn project_event_detail(event: &OperationEvent) -> OperationDetail {
    OperationDetail {
        id: event.id.clone(),
        operation: event.operation.clone(),
        kind: event.kind,
        status: event.status,
        namespace: event.namespace.clone(),
        summary: event.summary.clone(),
        payload: event.payload.clone(),
        read_only: true,
    }
}

fn count_kind(events: &[OperationEvent], kind: OperationKind) -> usize {
    events.iter().filter(|event| event.kind == kind).count()
}
```

Update `src/interfaces/dashboard/mod.rs`:

```rust
pub mod projection;
pub use projection::{
    build_summary, project_event_detail, DashboardRuntimeInfo, DashboardSummary, OperationDetail,
};
```

- [ ] **Step 4: Run tests**

Run:

```zsh
cargo test --test dashboard_projection -v
```

Expected: PASS.

- [ ] **Step 5: Commit**

```zsh
git add src/interfaces/dashboard/mod.rs src/interfaces/dashboard/projection.rs tests/dashboard_projection.rs
git commit -m "feat: add dashboard projections"
```

---

### Task 4: HTTP Service And Anime Dashboard Asset

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/interfaces/dashboard/mod.rs`
- Create: `src/interfaces/dashboard/assets.rs`
- Create: `src/interfaces/dashboard/http.rs`
- Create: `tests/dashboard_http.rs`

- [ ] **Step 1: Add failing HTTP tests**

Create `tests/dashboard_http.rs`:

```rust
use agent_llm_mm::{
    interfaces::dashboard::{
        start_dashboard_service, DashboardRuntimeInfo, EventQuery, OperationEvent, OperationKind,
        OperationRecorder, OperationStatus,
    },
    support::config::DashboardConfig,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dashboard_serves_html_summary_events_and_health() {
    let recorder = OperationRecorder::new(20);
    recorder.append(OperationEvent {
        id: "op_1".to_string(),
        sequence: 1,
        timestamp: Utc::now(),
        kind: OperationKind::Reflection,
        status: OperationStatus::Handled,
        operation: "auto_reflect".to_string(),
        namespace: Some("self".to_string()),
        summary: "proposal passed evidence gate".to_string(),
        correlation_id: None,
        payload: json!({ "reflection_id": "reflection-1" }),
    });

    let config = DashboardConfig {
        enabled: true,
        host: "127.0.0.1".to_string(),
        port: 0,
        base_path: "/".to_string(),
        event_capacity: 20,
        sse_enabled: true,
        open_browser: false,
        required: true,
    };
    let runtime = DashboardRuntimeInfo {
        service_name: "agent-llm-mm".to_string(),
        transport: "stdio".to_string(),
        provider: "mock".to_string(),
        dashboard_enabled: true,
        read_only: true,
    };

    let handle = start_dashboard_service(config, recorder.clone(), runtime)
        .await
        .expect("dashboard starts");
    let base_url = handle.base_url();
    let client = Client::new();

    let html = client
        .get(format!("{base_url}/"))
        .send()
        .await
        .expect("html response")
        .text()
        .await
        .expect("html body");
    assert!(html.contains("Memory-chan Live Cockpit"));

    let summary: serde_json::Value = client
        .get(format!("{base_url}/api/summary"))
        .send()
        .await
        .expect("summary response")
        .json()
        .await
        .expect("summary json");
    assert_eq!(summary["total_events"], 1);
    assert_eq!(summary["reflection_events"], 1);

    let events: serde_json::Value = client
        .get(format!("{base_url}/api/events?limit=5"))
        .send()
        .await
        .expect("events response")
        .json()
        .await
        .expect("events json");
    assert_eq!(events.as_array().expect("events array").len(), 1);

    let health: serde_json::Value = client
        .get(format!("{base_url}/api/health"))
        .send()
        .await
        .expect("health response")
        .json()
        .await
        .expect("health json");
    assert_eq!(health["status"], "ok");

    drop(handle);
    let _ = recorder.recent(EventQuery::default());
}
```

- [ ] **Step 2: Run test and verify it fails**

Run:

```zsh
cargo test --test dashboard_http -v
```

Expected: FAIL because `axum`, assets, and `start_dashboard_service` do not exist.

- [ ] **Step 3: Add dependencies**

In `Cargo.toml`, add:

```toml
axum = "0.8"
tokio-stream = { version = "0.1", features = ["sync"] }
```

- [ ] **Step 4: Add embedded dashboard HTML**

Create `src/interfaces/dashboard/assets.rs` with a compact version of the approved v3 UI:

```rust
pub const DASHBOARD_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Memory-chan Live Cockpit</title>
  <style>
    :root { color-scheme: light; --ink:#2a3146; --muted:#68758a; --pink:#ff8fb3; --sky:#78c7ff; --aqua:#5fe0cf; --lemon:#ffe37a; }
    * { box-sizing: border-box; }
    body { margin:0; font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color:var(--ink); background:linear-gradient(135deg, rgba(120,199,255,.22), transparent 30%), linear-gradient(225deg, rgba(255,143,179,.24), transparent 34%), #fbfdff; }
    @media (prefers-reduced-motion: no-preference) { .chibi { animation: floaty 3.2s ease-in-out infinite; } @keyframes floaty { 0%,100% { transform: translateY(0) rotate(-2deg); } 50% { transform: translateY(-7px) rotate(2deg); } } }
    .shell { min-height:100vh; padding:18px; }
    .stage { overflow:hidden; border:1px solid rgba(255,143,179,.28); border-radius:8px; background:rgba(255,255,255,.76); box-shadow:0 28px 72px rgba(55,65,92,.15); }
    .top { display:grid; grid-template-columns:310px 1fr auto; gap:16px; align-items:center; padding:16px 18px; border-bottom:1px solid rgba(42,49,70,.13); backdrop-filter:blur(16px); }
    .brand { display:flex; align-items:center; gap:12px; }
    .chibi { position:relative; width:54px; height:58px; border-radius:28px 28px 16px 16px; background:linear-gradient(145deg, var(--sky), #b7a3ff 50%, var(--pink)); box-shadow:inset 0 0 0 2px rgba(255,255,255,.74), 0 12px 24px rgba(183,163,255,.28); }
    .chibi::before { content:""; position:absolute; left:10px; right:10px; bottom:7px; height:30px; border-radius:18px 18px 12px 12px; background:#fff2ea; }
    .brand strong { display:block; font-size:20px; }
    .brand span { color:var(--muted); font-size:12px; }
    .ribbon { min-height:46px; display:flex; align-items:center; gap:10px; padding:0 12px; border:1px solid rgba(42,49,70,.12); border-radius:8px; background:repeating-linear-gradient(115deg, rgba(120,199,255,.16) 0 13px, rgba(255,255,255,.72) 13px 27px); overflow:hidden; }
    .live { border-radius:999px; padding:6px 10px; background:linear-gradient(135deg,var(--aqua),var(--sky)); color:#fff; font-weight:820; }
    .bubble,.pill { border:1px solid rgba(42,49,70,.11); border-radius:999px; padding:6px 9px; background:rgba(255,255,255,.72); color:#526175; font-size:12px; }
    .pills { display:flex; gap:7px; flex-wrap:wrap; justify-content:end; }
    .grid { display:grid; grid-template-columns:270px minmax(0,1fr) 320px; min-height:620px; }
    aside, main { padding:18px; }
    .left { border-right:1px solid rgba(42,49,70,.13); }
    .right { border-left:1px solid rgba(42,49,70,.13); }
    .label { margin-bottom:10px; color:#7a8698; font-size:11px; letter-spacing:.1em; text-transform:uppercase; }
    .tab,.score,.card,.panel,.story { border:1px solid rgba(42,49,70,.13); border-radius:8px; background:rgba(255,255,255,.86); }
    .tab { display:flex; align-items:center; gap:10px; min-height:42px; padding:0 12px; margin-bottom:8px; font-weight:650; color:#526175; }
    .tab.active { color:#8b3857; background:linear-gradient(90deg, rgba(255,143,179,.16), rgba(120,199,255,.14)); }
    .star { width:15px; height:15px; clip-path:polygon(50% 0,61% 34%,98% 35%,68% 55%,80% 91%,50% 70%,20% 91%,32% 55%,2% 35%,39% 34%); background:var(--lemon); }
    .score { padding:13px; margin-top:12px; }
    .score strong,.stat strong { font-size:26px; }
    .score span,.stat label { display:block; color:var(--muted); font-size:12px; }
    .meter { height:9px; border-radius:999px; margin-top:10px; background:linear-gradient(90deg,var(--aqua),var(--sky),var(--lemon),var(--pink)); }
    .stats { display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:12px; margin-bottom:14px; }
    .stat { padding:13px; min-height:76px; }
    .story { padding:14px; margin-bottom:14px; }
    .steps { display:grid; grid-template-columns:repeat(5,minmax(0,1fr)); gap:10px; }
    .step { min-height:82px; padding:12px; border:1px solid rgba(42,49,70,.1); border-radius:8px; background:rgba(255,255,255,.74); }
    .chip,.tag { display:inline-flex; border-radius:999px; padding:5px 8px; font-size:11px; font-weight:760; background:rgba(95,224,207,.18); color:#23655e; }
    .logs { display:grid; gap:10px; }
    .log { display:grid; grid-template-columns:92px minmax(120px,.7fr) 94px minmax(0,1fr) 84px; gap:12px; align-items:center; min-height:58px; padding:0 13px; font-size:12px; box-shadow:0 8px 18px rgba(55,65,92,.05); }
    .log.hot { border-color:rgba(255,143,179,.34); background:linear-gradient(90deg, rgba(255,143,179,.13), rgba(255,255,255,.86) 38%); }
    .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
    .panel { padding:13px; margin-bottom:12px; }
    .kv { display:grid; grid-template-columns:88px 1fr; gap:8px; padding:6px 0; border-top:1px solid rgba(42,49,70,.07); font-size:12px; }
    .kv:first-of-type { border-top:0; }
    pre { margin:0; padding:11px; border-radius:8px; background:#263244; color:#eaf0fb; overflow:auto; font-size:11px; }
    @media (max-width: 1160px) { .top,.grid { grid-template-columns:1fr; } .pills { justify-content:start; } .left,.right { border:0; } .stats { grid-template-columns:repeat(2,minmax(0,1fr)); } .steps { grid-template-columns:1fr; } .log { grid-template-columns:78px 1fr 88px; } .log div:nth-child(4),.log div:nth-child(5) { display:none; } }
  </style>
</head>
<body>
  <div class="shell">
    <div class="stage">
      <div class="top">
        <div class="brand"><div class="chibi"></div><div><strong>Memory-chan Live Cockpit</strong><span>agent-llm-mm operation diary</span></div></div>
        <div class="ribbon"><span class="live">LIVE</span><span class="bubble" id="live-operation">waiting for events</span><span class="bubble">read-only observability</span></div>
        <div class="pills"><span class="pill">stdio MCP</span><span class="pill">production dashboard</span><span class="pill">stdout protected</span></div>
      </div>
      <div class="grid">
        <aside class="left"><div class="label">Live channels</div><div class="tab active"><span class="star"></span> Operation Diary</div><div class="tab"><span class="star"></span> Reflection Magic</div><div class="tab"><span class="star"></span> Decision Cards</div><div class="tab"><span class="star"></span> Snapshot Album</div><div class="tab"><span class="star"></span> Doctor Check</div><div class="score"><strong id="total-score">0</strong><span>operations this session</span><div class="meter"></div></div><div class="score"><strong id="reflection-score">0</strong><span>reflection events</span><div class="meter"></div></div><div class="score"><strong id="failed-score">0</strong><span>failed operations</span><div class="meter"></div></div></aside>
        <main><div class="stats"><div class="card stat"><label>MCP tools</label><strong>4</strong></div><div class="card stat"><label>events</label><strong id="event-count">0</strong></div><div class="card stat"><label>reflections</label><strong id="reflection-count">0</strong></div><div class="card stat"><label>status</label><strong>ok</strong></div></div><section class="story"><div class="label">Operation story</div><div class="steps"><div class="step"><b>ingest</b><br><span class="chip">watching</span></div><div class="step"><b>trigger</b><br><span class="chip">watching</span></div><div class="step"><b>proposal</b><br><span class="chip">watching</span></div><div class="step"><b>write path</b><br><span class="chip">run_reflection</span></div><div class="step"><b>snapshot</b><br><span class="chip">watching</span></div></div></section><section class="logs" id="logs"></section></main>
        <aside class="right"><div class="panel"><h4>Selected operation</h4><div id="selected-operation"><div class="kv"><span>status</span><span>waiting</span></div></div></div><div class="panel"><h4>Payload inspector</h4><pre id="payload">{}</pre></div><div class="panel"><h4>Runtime note</h4><div class="kv"><span>mode</span><span>production dashboard</span></div><div class="kv"><span>actions</span><span>read only</span></div><div class="kv"><span>stdout</span><span>MCP only</span></div></div></aside>
      </div>
    </div>
  </div>
  <script>
    async function refresh() {
      const [summary, events] = await Promise.all([
        fetch('/api/summary').then(r => r.json()),
        fetch('/api/events?limit=25').then(r => r.json())
      ]);
      document.getElementById('total-score').textContent = summary.total_events;
      document.getElementById('reflection-score').textContent = summary.reflection_events;
      document.getElementById('failed-score').textContent = summary.failed_events;
      document.getElementById('event-count').textContent = summary.total_events;
      document.getElementById('reflection-count').textContent = summary.reflection_events;
      const logs = document.getElementById('logs');
      logs.innerHTML = events.map((event, index) => `<button class="card log ${index === events.length - 1 ? 'hot' : ''}" data-event='${JSON.stringify(event).replaceAll("'", "&#39;")}'><div class="mono">${new Date(event.timestamp).toLocaleTimeString()}</div><div>${event.operation}</div><div><span class="tag">${event.kind}</span></div><div>${event.summary}</div><div>${event.status}</div></button>`).join('');
      const latest = events[events.length - 1];
      if (latest) selectEvent(latest);
      logs.querySelectorAll('.log').forEach(node => node.addEventListener('click', () => selectEvent(JSON.parse(node.dataset.event))));
    }
    function selectEvent(event) {
      document.getElementById('live-operation').textContent = event.operation;
      document.getElementById('selected-operation').innerHTML = `<div class="kv"><span>id</span><span class="mono">${event.id}</span></div><div class="kv"><span>kind</span><span>${event.kind}</span></div><div class="kv"><span>status</span><span>${event.status}</span></div><div class="kv"><span>namespace</span><span>${event.namespace || '-'}</span></div>`;
      document.getElementById('payload').textContent = JSON.stringify(event.payload || {}, null, 2);
    }
    refresh();
    setInterval(refresh, 2500);
  </script>
</body>
</html>"#;
```

- [ ] **Step 5: Implement HTTP service**

Create `src/interfaces/dashboard/http.rs`:

```rust
use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        sse::{Event, Sse},
        Html, IntoResponse,
    },
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_stream::{wrappers::BroadcastStream, StreamExt};

use crate::support::config::DashboardConfig;

use super::{
    assets::DASHBOARD_HTML, build_summary, project_event_detail, DashboardRuntimeInfo, EventQuery,
    OperationKind, OperationRecorder, OperationStatus,
};

#[derive(Clone)]
struct DashboardState {
    recorder: OperationRecorder,
    runtime: DashboardRuntimeInfo,
}

pub struct DashboardHandle {
    address: SocketAddr,
    task: tokio::task::JoinHandle<()>,
}

impl DashboardHandle {
    pub fn base_url(&self) -> String {
        format!("http://{}", self.address)
    }
}

impl Drop for DashboardHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    limit: Option<usize>,
    kind: Option<OperationKind>,
    status: Option<OperationStatus>,
    namespace: Option<String>,
}

pub async fn start_dashboard_service(
    config: DashboardConfig,
    recorder: OperationRecorder,
    runtime: DashboardRuntimeInfo,
) -> Result<DashboardHandle> {
    config.validate().map_err(anyhow::Error::msg)?;
    let address = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&address).await?;
    let address = listener.local_addr()?;
    let state = DashboardState { recorder, runtime };
    let app = router(state);
    let task = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::warn!(error = %error, "dashboard service stopped with error");
        }
    });

    Ok(DashboardHandle { address, task })
}

fn router(state: DashboardState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/api/summary", get(summary))
        .route("/api/events", get(events))
        .route("/api/events/{id}", get(event_detail))
        .route("/api/events/stream", get(event_stream))
        .route("/api/health", get(health))
        .with_state(Arc::new(state))
}

async fn index() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn summary(State(state): State<Arc<DashboardState>>) -> Json<super::DashboardSummary> {
    let events = state.recorder.recent(EventQuery::default());
    Json(build_summary(&events, &state.runtime))
}

async fn events(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<EventsQuery>,
) -> Json<Vec<super::OperationEvent>> {
    Json(state.recorder.recent(EventQuery {
        limit: query.limit,
        kind: query.kind,
        status: query.status,
        namespace: query.namespace,
    }))
}

async fn event_detail(
    State(state): State<Arc<DashboardState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let events = state.recorder.recent(EventQuery::default());
    match events.iter().find(|event| event.id == id) {
        Some(event) => Json(project_event_detail(event)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "operation event not found" })),
        )
            .into_response(),
    }
}

async fn event_stream(
    State(state): State<Arc<DashboardState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(state.recorder.subscribe()).filter_map(|event| match event {
        Ok(event) => Some(Ok(Event::default().json_data(event).unwrap_or_else(|_| {
            Event::default().data(r#"{"error":"failed to serialize dashboard event"}"#)
        }))),
        Err(_) => None,
    });
    Sse::new(stream)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "read_only": true }))
}
```

Update `src/interfaces/dashboard/mod.rs`:

```rust
pub mod assets;
pub mod http;
pub use http::{start_dashboard_service, DashboardHandle};
```

- [ ] **Step 6: Run HTTP test**

Run:

```zsh
cargo test --test dashboard_http -v
```

Expected: PASS.

- [ ] **Step 7: Commit**

```zsh
git add Cargo.toml src/interfaces/dashboard/assets.rs src/interfaces/dashboard/http.rs src/interfaces/dashboard/mod.rs tests/dashboard_http.rs
git commit -m "feat: serve dashboard http api"
```

---

### Task 5: MCP Runtime Observer Integration

**Files:**
- Modify: `src/interfaces/dashboard/mod.rs`
- Modify: `src/interfaces/dashboard/event.rs`
- Modify: `src/interfaces/mcp/server.rs`
- Modify: `tests/mcp_stdio.rs`

- [ ] **Step 1: Add failing MCP stdout safety test**

Append to `tests/mcp_stdio.rs`:

```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve port");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    let config = r#"
transport = "stdio"
database_url = "__DATABASE_URL__"

[dashboard]
enabled = true
host = "127.0.0.1"
port = __DASHBOARD_PORT__
event_capacity = 50
required = true
"#
    .replace("__DASHBOARD_PORT__", &port.to_string());
    let mut client = test_support::spawn_stdio_client_with_config(config)
        .await
        .expect("client");

    let tools = client.list_all_tools().await.expect("list tools");
    assert_eq!(tools.len(), 4);

    let health: serde_json::Value = reqwest::get(format!("http://127.0.0.1:{port}/api/health"))
        .await
        .expect("dashboard health response")
        .json()
        .await
        .expect("dashboard health json");
    assert_eq!(health["status"], "ok");

    let response = client
        .call_tool(
            "build_self_snapshot",
            json!({
                "budget": 10
            }),
        )
        .await
        .expect("snapshot response");

    assert!(
        response.get("result").is_some(),
        "dashboard logs must not corrupt MCP stdout: {response:?}"
    );
}
```

- [ ] **Step 2: Run test and verify it fails**

Run outside sandbox because this test starts a local dashboard listener:

```zsh
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

Expected: FAIL because `dashboard.enabled` is parsed after Task 1 but `serve` does not start dashboard.

- [ ] **Step 3: Add event normalization helpers**

In `src/interfaces/dashboard/event.rs`, add:

```rust
use uuid::Uuid;

pub fn tool_started(operation: &str, namespace: Option<String>, sequence: u64) -> OperationEvent {
    OperationEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Tool,
        status: OperationStatus::Started,
        operation: operation.to_string(),
        namespace,
        summary: format!("{operation} started"),
        correlation_id: None,
        payload: serde_json::json!({ "operation": operation }),
    }
}

pub fn tool_completed(
    operation_id: String,
    operation: &str,
    namespace: Option<String>,
    sequence: u64,
    summary: String,
    payload: serde_json::Value,
) -> OperationEvent {
    OperationEvent {
        id: operation_id,
        sequence,
        timestamp: Utc::now(),
        kind: OperationKind::Tool,
        status: OperationStatus::Ok,
        operation: operation.to_string(),
        namespace,
        summary,
        correlation_id: None,
        payload,
    }
}
```

Add similar `tool_failed` and `auto_reflection_event` helpers with `OperationStatus::Failed`, `Handled`, `Suppressed`, or `Rejected` based on diagnostics.

- [ ] **Step 4: Add dashboard observer wrapper**

In `src/interfaces/dashboard/mod.rs`, add:

```rust
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use serde::Serialize;
use serde_json::json;

#[derive(Clone)]
pub enum DashboardObserver {
    Disabled,
    Enabled {
        recorder: OperationRecorder,
        sequence: Arc<AtomicU64>,
    },
}

impl DashboardObserver {
    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub fn enabled(recorder: OperationRecorder) -> Self {
        Self::Enabled {
            recorder,
            sequence: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn recorder(&self) -> Option<OperationRecorder> {
        match self {
            Self::Disabled => None,
            Self::Enabled { recorder, .. } => Some(recorder.clone()),
        }
    }

    pub fn next_sequence(&self) -> u64 {
        match self {
            Self::Disabled => 0,
            Self::Enabled { sequence, .. } => sequence.fetch_add(1, Ordering::Relaxed) + 1,
        }
    }

    pub fn record_event(&self, event: OperationEvent) {
        if let Self::Enabled { recorder, .. } = self {
            recorder.append(event);
        }
    }

    pub fn record_tool_ok<T: Serialize>(
        &self,
        operation: &str,
        namespace: Option<String>,
        summary: String,
        payload: &T,
    ) {
        let sequence = self.next_sequence();
        if sequence == 0 {
            return;
        }
        self.record_event(OperationEvent {
            id: uuid::Uuid::new_v4().to_string(),
            sequence,
            timestamp: chrono::Utc::now(),
            kind: OperationKind::Tool,
            status: OperationStatus::Ok,
            operation: operation.to_string(),
            namespace,
            summary,
            correlation_id: None,
            payload: serde_json::to_value(payload).unwrap_or_else(|_| json!({ "serialization": "failed" })),
        });
    }
}
```

- [ ] **Step 5: Start dashboard from serve path**

In `src/interfaces/mcp/server.rs`:

- Import dashboard types.
- In `run_stdio_server_with_config`, if `config.dashboard.enabled`, create an `OperationRecorder`, start the HTTP service, log the URL with `tracing::info!`, and pass `DashboardObserver::enabled(recorder)` into `Server::from_config`.
- If startup fails and `required = true`, return the error.
- If startup fails and `required = false`, log warning and use `DashboardObserver::disabled()`.
- Change `Server::from_config(config)` to `Server::from_config(config, dashboard_observer)`.
- Add `dashboard: DashboardObserver` to `Runtime`.
- After each successful tool call, call `self.runtime.dashboard.record_tool_ok(...)`.
- On mapped app errors, record failed events before returning the MCP error.

Minimal success recording examples:

```rust
self.runtime.dashboard.record_tool_ok(
    "ingest_interaction",
    Some(result.owner_namespace.to_string()),
    format!("ingest stored event {}", result.event_id),
    &result,
);
```

For result types without direct namespace fields, use `None` or derive from request params before conversion.

- [ ] **Step 6: Run targeted MCP test**

Run:

```zsh
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

Expected: PASS.

- [ ] **Step 7: Run existing MCP tests**

Run:

```zsh
cargo test --test mcp_stdio -v
```

Expected: PASS.

- [ ] **Step 8: Commit**

```zsh
git add src/interfaces/dashboard/mod.rs src/interfaces/dashboard/event.rs src/interfaces/mcp/server.rs tests/mcp_stdio.rs
git commit -m "feat: record mcp operations for dashboard"
```

---

### Task 6: Docs And Example Config

**Files:**
- Modify: `README.md`
- Modify: `docs/project-status.md`
- Modify: `docs/development-macos.md`
- Modify: `docs/testing-guide-2026-03-24.md`
- Modify: `examples/agent-llm-mm.example.toml`

- [ ] **Step 1: Update example config**

Append to `examples/agent-llm-mm.example.toml`:

```toml
[dashboard]
enabled = true
host = "127.0.0.1"
port = 8787
base_path = "/"
event_capacity = 2000
sse_enabled = true
open_browser = false
required = false
```

- [ ] **Step 2: Update README capability list**

In `README.md`, add a new implemented capability under “当前能力 / 已实现”:

```markdown
- production dashboard service
  - 可通过 `[dashboard]` 配置启停
  - 随 `serve` 启动本机 HTTP 只读观测面板
  - 展示 MCP tool 调用、runtime operation、reflection / decision / snapshot 事件
  - 不改变 MCP tool 列表，不污染 MCP `stdout`
```

- [ ] **Step 3: Update project status**

In `docs/project-status.md`, add the dashboard service to implemented or partial status depending on final coverage. Use this wording after Task 5 passes:

```markdown
### Production dashboard service

- 已支持通过 `[dashboard]` 配置随 `serve` 启动只读 HTTP 面板
- 面板展示运行时 operation 事件，并保持 MCP `stdio` 输出不被污染
- 当前事件记录为 bounded in-memory recorder，不是 durable operation-log database
```

- [ ] **Step 3a: Update asset notice**

In `NOTICE`, record the dashboard generated image assets:

```text
src/interfaces/dashboard/static/memory_chan_hero.png
src/interfaces/dashboard/static/memory_chan_sidebar.png
```

The notice should state that these are project-specific generated assets for
`Memory-chan Live Cockpit`, not third-party stock artwork.

- [ ] **Step 4: Update macOS development guide**

In `docs/development-macos.md`, add a short dashboard section:

```markdown
## Dashboard 面板

在 `agent-llm-mm.local.toml` 中启用：

```toml
[dashboard]
enabled = true
host = "127.0.0.1"
port = 8787
```

然后启动：

```zsh
./scripts/agent-llm-mm.sh serve agent-llm-mm.local.toml
```

浏览器访问 `http://127.0.0.1:8787/`。该面板只读，不会调用 `run_reflection` 或修改 SQLite。
```

- [ ] **Step 5: Update testing guide**

In `docs/testing-guide-2026-03-24.md`, add dashboard verification commands:

```markdown
如果改动涉及 dashboard：

```zsh
cargo test --test dashboard_config --test dashboard_recorder --test dashboard_projection --test dashboard_http
cargo test --test mcp_stdio dashboard_enabled_does_not_corrupt_mcp_stdout_and_records_tool_event -v
```

dashboard HTTP 测试会监听本机端口，受限沙箱中可能需要在允许本地监听的环境运行。
```

- [ ] **Step 6: Run doc-oriented checks**

Run:

```zsh
rg -n "dashboard|Memory-chan|8787" README.md docs examples
```

Expected: Entries appear in README, project status, macOS guide, testing guide, and example config.

- [ ] **Step 7: Commit**

```zsh
git add README.md docs/project-status.md docs/development-macos.md docs/testing-guide-2026-03-24.md examples/agent-llm-mm.example.toml
git commit -m "docs: document dashboard service"
```

---

### Task 7: Final Verification

**Files:**
- No code files should be edited in this task unless verification finds a defect.

- [ ] **Step 1: Format**

Run:

```zsh
cargo fmt --check
```

Expected: PASS.

- [ ] **Step 2: Clippy**

Run:

```zsh
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS.

- [ ] **Step 3: Full tests**

Run outside the restricted sandbox because dashboard and existing stub tests bind local ports:

```zsh
cargo test
```

Expected: PASS with all tests passing.

- [ ] **Step 4: Doctor**

Run:

```zsh
./scripts/agent-llm-mm.sh doctor
```

Expected: JSON report includes `status = "ok"` and the new dashboard config fields.

- [ ] **Step 5: Manual dashboard smoke**

Create a temporary config:

```zsh
cat > /tmp/agent-llm-mm-dashboard-smoke.toml <<'EOF'
transport = "stdio"
database_url = "sqlite:///tmp/agent-llm-mm-dashboard-smoke.sqlite"

[dashboard]
enabled = true
host = "127.0.0.1"
port = 8787
event_capacity = 2000
required = true
EOF
```

Start the service:

```zsh
./scripts/agent-llm-mm.sh serve /tmp/agent-llm-mm-dashboard-smoke.toml
```

In another terminal or test command, verify:

```zsh
curl -s http://127.0.0.1:8787/api/health
curl -s http://127.0.0.1:8787/ | rg "Memory-chan Live Cockpit"
```

Expected: health returns `{"status":"ok","read_only":true}` and HTML contains `Memory-chan Live Cockpit`.

- [ ] **Step 6: Git status**

Run:

```zsh
git status --short --branch
```

Expected: clean working tree on `codex/dashboard-production-service`.
