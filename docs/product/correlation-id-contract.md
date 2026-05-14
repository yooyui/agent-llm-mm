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
the matching dashboard event or operation-log row. A support bundle must still
redact secrets and exclude full SQLite databases by default.

## Verification

Relevant tests:

```bash
cargo test --test dashboard_projection --test mcp_stdio --test operation_log -v
```

Expected coverage:

- dashboard detail projection preserves `correlation_id`
- dashboard events record generated MCP correlation IDs
- distinct MCP calls receive distinct correlation IDs
- MCP tool calls append runtime operation-log entries with correlation IDs
- operation-log queries by `correlation_id` continue to work
