# Correlation ID Contract

## Scope

Correlation IDs make Local Alpha debugging traceable across MCP tool calls,
dashboard events, runtime operation-log entries, and best-effort
auto-reflection diagnostics.

This contract is observability metadata only. It does not create a new durable
identity, commitment, claim, or reflection write path. Identity and commitment
updates still flow only through governed `run_reflection`.

## Format

When a caller omits a correlation ID, the MCP server generates one per
`tools/call` handler invocation:

```text
mcp-tool-call-<uuid-v4>
```

The value is opaque. Consumers may use it for equality, filtering, and support
bundle summaries, but must not parse semantic meaning from the UUID.

## Source and Propagation

Current Local Alpha behavior:

- every MCP tool handler generates a fresh correlation ID at the start of the
  call
- successful dashboard tool events include the generated correlation ID
- failed dashboard tool events include the same generated correlation ID
- best-effort auto-reflection dashboard events reuse the same correlation ID as
  the triggering MCP tool call
- successful MCP tool calls append runtime `operation_log` metadata with the
  generated correlation ID
- dashboard detail projection exposes `correlation_id` for in-memory events and
  durable operation-log projections

This first slice does not expose a public tool parameter for caller-provided
correlation IDs. If a future client-supplied value is needed, it must be added as
an explicit contract with validation and tests.

## Coverage Matrix

| Surface | Status | Current propagation | Verification |
| --- | --- | --- | --- |
| MCP call | `implemented` | Project-level `tools/call` handlers generate one `mcp-tool-call-<uuid-v4>` correlation ID per handler invocation. | `tests/mcp_stdio.rs` covers successful calls, handler-reached failures, and distinct generated IDs. |
| Dashboard event | `implemented` | Successful and failed tool events include the generated MCP correlation ID; best-effort auto-reflection dashboard events reuse the triggering tool call correlation ID. | `tests/mcp_stdio.rs` and `tests/dashboard_http.rs` cover dashboard events and read-only operation-log history projection. |
| Operation-log entry | `implemented` | Handler-level success and handler-reached failure entries persist the generated correlation ID as safe observability metadata. | `tests/mcp_stdio.rs` covers success and failure persistence; `tests/operation_log.rs` covers correlation filtering. |
| Support bundle operation summary | `implemented` | Explicit `--correlation-id mcp-tool-call-<uuid-v4>` filters operation-log metadata rows and keeps payload summaries omitted. | `tests/support_bundle.rs` covers canonical filter validation, redaction, bounds, and script forwarding. |
| Framework-level MCP parse / route failure | `documented gap` | Non-object arguments and other framework-level failures can fail before the project handler generates durable operation-log metadata. | `tests/mcp_stdio.rs` documents the boundary with a no-entry assertion. |
| Model call | `future` | Provider calls do not yet carry a cross-surface correlation ID contract. Provider errors are summarized safely at the MCP operation boundary only. | Raw provider payload redaction is covered, but model-call correlation is not implemented. |
| Trigger ledger entry | `future` | Trigger ledger rows do not yet persist a correlation ID field. Auto-reflection diagnostics may be emitted to dashboard with the tool correlation ID. | Runtime hook diagnostics are tested separately; ledger correlation is not claimed. |
| Reflection audit entry | `future` | Reflection audit rows are still governed by `run_reflection` and do not yet persist MCP correlation ID metadata. | Reflection write governance is tested; reflection correlation is not claimed. |

## Operation Log Boundary

The runtime operation log stores tool-level observability metadata:

- `entrypoint`
- `operation_kind = tool`
- `status`
- `namespace`
- generated `correlation_id`
- redacted request / response / diagnostic summaries when available

This is not a semantic memory write. It must not bypass `run_reflection` for
identity, commitment, claim, or reflection changes.

## Support Bundle Use

Support bundle summaries may include correlation IDs so a maintainer can ask for
the matching dashboard event or operation-log row. A support bundle may also be
generated with an explicit `--correlation-id <id>` filter to include only
matching operation-log metadata rows in `operation-summaries.json`.

The support bundle filter accepts only the canonical generated
`mcp-tool-call-<uuid-v4>` shape. This keeps arbitrary user text, provider
tokens, or secret-like values out of the bundle metadata. Filtered operation
summaries remain metadata-only and must still redact secrets, omit request /
response / diagnostic payload summaries, avoid database migration, and exclude
full SQLite databases by default.

## Verification

Relevant tests:

```bash
cargo test --test dashboard_projection --test mcp_stdio --test operation_log -v
cargo test --test support_bundle -v
```

Expected coverage:

- dashboard detail projection preserves `correlation_id`
- dashboard events record generated MCP correlation IDs
- distinct MCP calls receive distinct correlation IDs
- MCP tool calls append runtime operation-log entries with correlation IDs
- operation-log queries by `correlation_id` continue to work
- support bundle operation summaries can be filtered by generated
  `mcp-tool-call-<uuid-v4>` correlation IDs without exposing raw operation payloads
