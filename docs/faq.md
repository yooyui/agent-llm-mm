# FAQ

## What is MCP Memory Ledger?

MCP Memory Ledger is a local-first MCP `stdio` memory server for AI agents. It stores interaction and evidence history in SQLite, builds self snapshots, and supports governed self-revision through explicit evidence and bounded triggers.

## Why do some files still say `agent_llm_mm`?

MCP Memory Ledger is the project name. `agent_llm_mm` is the current implementation package name used by the Rust crate, binary, scripts, configuration examples, and historical documentation. It remains for compatibility until a separate technical rename migration is planned.

## Is this a vector database?

No. The current MVP is not a vector database. It focuses on durable interaction records, evidence events, snapshots, and conservative self-revision paths.

## Does it replace a general agent memory platform?

No. MCP Memory Ledger focuses on a narrower local-first MCP memory and evidence-gated self-revision loop. It should be described as a technical MVP, not a broad replacement for general memory platforms, graph memory engines, or stateful agent platforms.

## Does it run as a remote service?

Not as a production remote service. The current project is a local MCP `stdio` demo / MVP. Remote/team features remain explicitly gated and should not be claimed as implemented product capability.

## What does evidence-gated self-revision mean?

It means durable claim changes should be tied to explicit evidence events and controlled by bounded trigger, diagnostic, and rejection rules. The project avoids treating every model suggestion as an authoritative memory update.

## Is it production ready?

No. The MVP has local verification and release-gate artifacts, but the repository still documents missing product gates such as real fresh-machine evidence, Windows parity, packaging, release decision, remote/team boundaries, and security/auth gates.

## Who should use it now?

Use it for local AI client experiments, MCP memory integration research, self-agent memory prototypes, and Rust + SQLite MCP reference work.
