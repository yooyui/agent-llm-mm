# Dashboard Production Service Design (2026-04-25)

## 1. Summary

This design adds a production-oriented, read-only dashboard service for `agent-llm-mm`.

The dashboard starts with `agent_llm_mm serve` when enabled in configuration. It shows runtime operations, MCP tool calls, self-revision triggers, reflection diagnostics, decisions, snapshots, and service health in a fresh anime-style UI. It does not change the MCP tool contract, does not write memory, and does not print dashboard output to MCP `stdout`.

The repository can still be described conservatively as a Rust MCP `stdio` memory project unless a broader product-positioning change is requested. This design only makes the dashboard module itself production-oriented.

## 2. Goals

- Provide a production service panel that can be enabled or disabled by configuration.
- Keep MCP `stdio` protocol output clean. Dashboard URLs, warnings, and logs must not be written to `stdout`.
- Make the dashboard read-only. It can inspect runtime evidence, but it cannot call `run_reflection`, mutate SQLite, or edit provider configuration.
- Use a function-oriented service design: pure functions for event normalization, filtering, summary projection, and detail projection; side-effecting functions only at the HTTP, recorder, and runtime boundary.
- Preserve the approved visual direction: cute, fresh, lively anime style with the `Memory-chan Live Desk` concept.
- Keep runtime coupling narrow. MCP tool handlers should record operation events through a small observer interface and should not depend on HTML or HTTP details.

## 3. Non-Goals

- No new MCP tool.
- No writable dashboard actions.
- No authentication system in the first implementation. The config shape should leave room for `auth_token` and `allow_origins`, but this round should not ship a half-finished security layer.
- No external frontend build pipeline. The initial HTML/CSS/JS should be served as embedded/static assets from Rust.
- No durable operation-log database in the first implementation. Runtime events live in a bounded in-memory recorder. Durable log storage can be added later if needed.
- No automatic system browser launch by default.

## 4. Configuration

Add a dashboard config section to `AppConfig`.

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

### Field Semantics

- `enabled`
  - `false`: do not start the dashboard service.
  - `true`: start the dashboard with `serve`.
- `host`
  - Bind address. Default should be `127.0.0.1`.
  - Production operators can explicitly set `0.0.0.0` if they understand the exposure.
- `port`
  - Fixed port for production use.
  - `0` means ask the OS for an available port, useful in development and tests.
- `base_path`
  - HTTP path prefix. Default `/`.
  - Allows future reverse-proxy mounting such as `/agent-llm-mm/`.
- `event_capacity`
  - Ring-buffer event capacity. Default `2000`.
  - Must be greater than `0`.
- `sse_enabled`
  - Enables or disables the live event stream endpoint.
- `open_browser`
  - Default `false`.
  - If later implemented, it must never affect MCP stdout and should be documented as local developer convenience.
- `required`
  - `true`: dashboard startup failure fails `serve`.
  - `false`: dashboard startup failure records a warning and `serve` continues.

### Doctor Report

`doctor` should validate dashboard configuration but should not start the HTTP service.

The doctor report should include:

- `dashboard_enabled`
- `dashboard_host`
- `dashboard_port`
- `dashboard_base_path`
- `dashboard_required`

The report must not expose secrets if later dashboard auth fields are added.

## 5. Functional Service Architecture

Add a focused dashboard module:

```text
src/interfaces/dashboard/
  mod.rs
  config.rs
  event.rs
  recorder.rs
  projection.rs
  http.rs
  assets.rs
```

### Responsibilities

- `config.rs`
  - Defines `DashboardConfig` and validation helpers.
  - Converts from file config to runtime config through `AppConfig`.
- `event.rs`
  - Defines `OperationEvent`, `OperationKind`, `OperationStatus`, `OperationPayload`, and event normalization functions.
- `recorder.rs`
  - Owns the bounded in-memory event recorder.
  - Exposes append and query operations.
  - Provides a broadcast channel for SSE when enabled.
- `projection.rs`
  - Builds `DashboardSummary`, `OperationDetail`, and filtered event lists from recorder state.
  - Contains pure functions for counts, recent-event projections, and UI status models.
- `http.rs`
  - Starts/stops the dashboard HTTP service.
  - Serves HTML, JSON APIs, and optional SSE.
- `assets.rs`
  - Embeds the v3 anime dashboard HTML/CSS/JS.

### Function-Oriented Boundaries

Pure or mostly pure functions:

```rust
fn normalize_tool_started(input: ToolStartedInput) -> OperationEvent;
fn normalize_tool_completed(input: ToolCompletedInput) -> OperationEvent;
fn normalize_tool_failed(input: ToolFailedInput) -> OperationEvent;
fn normalize_auto_reflection(input: AutoReflectionInput) -> OperationEvent;
fn build_summary(events: &[OperationEvent], runtime: &RuntimeInfo) -> DashboardSummary;
fn filter_events(events: &[OperationEvent], query: &EventQuery) -> Vec<OperationEvent>;
fn project_event_detail(event: &OperationEvent) -> OperationDetail;
fn render_dashboard_html(config: &DashboardConfig) -> &'static str;
```

Side-effecting boundary functions:

```rust
async fn start_dashboard_service(
    config: DashboardConfig,
    recorder: OperationRecorder,
    shutdown: DashboardShutdown,
) -> Result<DashboardHandle>;

fn append_event(recorder: &OperationRecorder, event: OperationEvent);
```

MCP tool handlers should only call small recording helpers, for example:

```rust
let operation_id = self.runtime.dashboard.record_tool_started("ingest_interaction", &params);
...
self.runtime.dashboard.record_tool_completed(operation_id, &result);
```

The dashboard recording helper may be a no-op when `dashboard.enabled = false`.

## 6. Runtime Data Model

### OperationEvent

An event should be compact and serializable:

```rust
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
    pub payload: serde_json::Value,
}
```

### OperationKind

Suggested values:

- `startup`
- `tool`
- `trigger`
- `reflection`
- `decision`
- `snapshot`
- `doctor`
- `error`

### OperationStatus

Suggested values:

- `started`
- `ok`
- `handled`
- `suppressed`
- `rejected`
- `failed`

### Payload Boundaries

Payloads should be useful but conservative:

- Include non-secret operation metadata.
- Include IDs such as event id, reflection id, trigger key, and evidence event ids when available.
- Include error messages when they are already safe to expose in local logs.
- Do not include provider API keys.
- Do not include full prompts or user input by default if that could leak sensitive data. If a future version needs body capture, it should be separately configurable.

## 7. HTTP API

Initial endpoints:

- `GET /`
  - Returns the embedded dashboard HTML.
- `GET /api/summary`
  - Returns dashboard summary, counts, runtime status, and config-derived display fields.
- `GET /api/events?limit=100&kind=reflection&status=handled&namespace=self`
  - Returns recent filtered events.
- `GET /api/events/{id}`
  - Returns projected event detail.
- `GET /api/events/stream`
  - SSE stream for newly appended events when `sse_enabled = true`.
- `GET /api/health`
  - Returns dashboard service health.

All endpoints are read-only.

## 8. UI Design

Use the approved v3 direction:

- Product concept: `Memory-chan Live Desk`.
- Tone: cute, fresh, energetic anime dashboard.
- Layout:
  - Top: chibi assistant, live ribbon, current operation bubbles, runtime pills.
  - Left: live channels.
  - Center: status cards, operation story, card-style operation diary.
  - Right: selected operation, payload inspector, read-only runtime note.
- Channels:
  - `Operation Diary`
  - `Reflection Magic`
  - `Decision Cards`
  - `Snapshot Album`
  - `Doctor Check`
  - `Runtime Health`
- Visual elements:
  - Chibi assistant.
  - Sticker-like cards.
  - Candy-color meters.
  - Star status symbols.
  - Light pattern drift and sweep animations.
  - Reduced-motion CSS fallback through `prefers-reduced-motion`.

The UI should be cute without becoming hard to operate. Text must remain readable on desktop and mobile viewports.

## 9. Serve Lifecycle

`agent_llm_mm serve` should behave as follows:

1. Load `AppConfig`.
2. Validate provider, SQLite, and dashboard config.
3. Bootstrap the MCP runtime.
4. If dashboard is enabled, create `OperationRecorder` and start `DashboardServer`.
5. Record a `startup` event after successful dashboard bootstrap.
6. Start MCP `stdio` server.
7. On process shutdown, stop the dashboard server together with the MCP service.

If dashboard startup fails:

- `required = true`: return an error and fail `serve`.
- `required = false`: log a warning through tracing/stderr and continue serving MCP.

## 10. MCP Integration Points

Record events around these existing tool handlers:

- `ingest_interaction`
  - started / ok / failed.
  - auto-reflection diagnostics when present.
- `build_self_snapshot`
  - started / ok / failed.
  - periodic auto-reflection diagnostics when present.
- `decide_with_snapshot`
  - started / ok / failed.
  - decision blocked status and action summary.
  - conflict auto-reflection diagnostics when present.
- `run_reflection`
  - started / ok / failed.
  - reflection id and evidence ids when present.

The event recorder must not change MCP responses. Best-effort dashboard recording failures should not alter successful MCP tool calls.

## 11. Error Handling

- Invalid dashboard config should fail config loading or validation with a clear message.
- Fixed-port conflicts should follow `required`.
- SSE client disconnects should be ignored.
- Recorder overflow should drop the oldest event.
- Dashboard APIs should return structured JSON errors.
- HTML asset serving errors should return HTTP 500 and should be covered by tests.

## 12. Testing Plan

Implementation should use TDD.

### Config Tests

- Default dashboard config is parsed.
- TOML overrides all dashboard fields.
- `event_capacity = 0` is invalid.
- Invalid `base_path` is rejected.
- Doctor reports dashboard config and does not start HTTP.

### Recorder Tests

- Events append in sequence order.
- Ring buffer drops oldest events after capacity.
- Filtering by kind/status/namespace works.
- SSE broadcast is optional and does not block append.

### Projection Tests

- Summary counts operation kinds and statuses correctly.
- Recent event projection preserves newest ordering.
- Detail projection includes payload inspector fields.

### HTTP Tests

- `GET /` returns HTML containing `Memory-chan Live Desk`.
- `GET /api/summary` returns JSON summary.
- `GET /api/events` returns recent events.
- `GET /api/health` returns healthy state.

### MCP / Stdout Tests

- With dashboard enabled, MCP `stdout` remains valid JSON-RPC only.
- Tool calls append dashboard operation events.
- Dashboard disabled path remains a no-op and does not change existing behavior.

## 13. Documentation Updates

Update:

- `README.md`
- `docs/project-status.md`
- `docs/development-macos.md`
- `docs/testing-guide-2026-03-24.md`
- `examples/agent-llm-mm.example.toml`

Update `examples/codex-mcp-config.toml` only if the MCP registration flow changes. The initial dashboard service should not require a Codex MCP config change.

## 14. Implementation Decisions

These decisions are fixed for the first implementation:

- HTTP stack: use `axum` on the existing `tokio` runtime. It provides clear JSON routes and SSE support without adding a frontend build pipeline.
- Startup location: `interfaces::mcp::run_stdio_server_with_config` starts the dashboard service before starting MCP `stdio`, then passes a dashboard recorder/observer into `Server::from_config`. The dashboard implementation remains isolated under `src/interfaces/dashboard/`.
- Default enablement: `dashboard.enabled` defaults to `false` for production safety. The example config should show how to set it to `true` so the panel starts with `serve`.
- Payload capture: do not expose full request params by default. Payloads include safe summaries, operation names, namespace, IDs, status, trigger diagnostics, and error text already suitable for local logs.
- Browser launch: `open_browser` remains parsed but initially has no side effect unless implemented safely for local developer use. Production behavior must not auto-open browsers.

## 15. Acceptance Criteria

- `serve` can start dashboard when `dashboard.enabled = true`.
- `serve` does not start dashboard when `dashboard.enabled = false`.
- Dashboard URL/logging never corrupts MCP `stdout`.
- Dashboard renders the approved cute anime UI.
- Dashboard shows live operation records for existing MCP tools.
- Dashboard is read-only.
- Tests cover config, recorder, projection, HTTP smoke behavior, and MCP stdout safety.
- Docs and example config explain how to enable, disable, and configure the dashboard.
