# MCP Memory Ledger

SQLite と証拠ベースの自己修正で支える、ローカル AI Agent 向け MCP memory layer。

Languages: [English](../README.md) | [简体中文](README.zh-CN.md) | 日本語

MCP Memory Ledger は、ローカル AI クライアント向けの Rust 製 MCP `stdio` memory service です。interaction、evidence、claim、self snapshot、reflection audit を SQLite に保存し、Agent が一回限りの prompt context だけに依存せず、明示的で監査可能な境界の中で長期記憶を扱えるようにします。

現在のプロジェクトは、ローカル Agent memory、MCP integration、SQLite persistence、governed self-revision のための technical MVP です。production-grade autonomous-agent platform ではありません。remote team mode、multi-tenancy、packaged installers、daemon write capabilities、production security boundaries は、今後の roadmap / gate 対象です。

## Features

- **Local MCP memory service**: MCP `stdio` 経由で `ingest_interaction`、`build_self_snapshot`、`decide_with_snapshot`、`run_reflection` を提供します。
- **SQLite persistence**: event、claim、evidence、reflection audit、trigger ledger、operation log を保存します。
- **Evidence-gated self-revision**: claim、identity、commitment の更新には明示的な evidence と governance rule が必要です。identity / commitment / reflection の永続化 write path は `run_reflection` のみです。
- **Observable local diagnostics**: read-only dashboard、`doctor` preflight、operation-log lookup、redacted support-bundle generator を含みます。
- **Provider integration**: `mock`、`openai-compatible`、OpenRouter の config path をサポートします。provider secrets は private local config または environment variables に置く前提です。
- **Local release gates**: local alpha evidence、provider preflight、packaging preflight、release evidence summary の read-only checks を含みます。これらは現在の境界を証明するものであり、release の公開や認証は行いません。

## Use Cases

- ローカル AI client に MCP memory を追加する。
- Agent が explicit evidence に基づいて long-term memory を更新する方法を検証する。
- self snapshot、reflection、commitment gate の最小ループを確認する。
- Rust + SQLite + MCP `stdio` project の engineering reference として使う。
- provider、config、dashboard、operation-log の問題調査に使う local diagnostic material を生成する。

## Quick Start

macOS:

```zsh
./scripts/agent-llm-mm.sh bootstrap-local
./scripts/agent-llm-mm.sh doctor
./scripts/agent-llm-mm.sh serve
```

Windows:

```powershell
pwsh -File .\scripts\agent-llm-mm.ps1 bootstrap-local
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
pwsh -File .\scripts\agent-llm-mm.ps1 serve
```

`bootstrap-local` は development example から local config template を作成します。既存ファイルを上書きせず、secret を生成せず、service も起動しません。

Platform and integration guides:

- [macOS development guide](development-macos.md)
- [Windows development guide](development-windows.md)
- [Local MCP integration guide](local-mcp-integration-2026-03-26.md)

## Demo

再現可能な self-revision demo を実行します。

```zsh
./scripts/run-self-revision-demo.sh
```

この demo は deterministic local `openai-compatible` stub provider を起動し、real MCP `stdio` service を通して canonical scenario を実行し、report を `target/reports/self-revision-demo/...` に出力します。

## Local Diagnostics

Redacted support bundle を生成します。

```zsh
./scripts/generate-support-bundle.sh target/support-bundles/manual-check
```

support bundle には redacted JSON summary だけが含まれます。full SQLite database、raw TOML、provider payload、raw `.log` files はコピーしません。特定の log excerpt または MCP tool call を出力する場合は、`--log-file` または `--correlation-id` を明示してください。

Local alpha gate evidence を要約します。

```zsh
./scripts/local-alpha-evidence-summary.sh \
  --evidence-root . \
  --output-json target/reports/local-alpha/evidence-summary.json \
  --output-md target/reports/local-alpha/evidence-summary.md
```

この command は既存の local evidence を読み、summary を生成するだけです。不足している evidence を作成せず、local alpha readiness も認証しません。

## Current Boundaries

Implemented:

- MCP `stdio` main flow
- SQLite persistence with owner / namespace constraints
- Audited claim replacement through `run_reflection`
- Minimal identity and commitment revision
- Trigger-ledger-backed automatic self-revision MVP
- Read-only dashboard, `doctor`, local support bundle, and local gate summary scripts

Partially implemented:

- `decide_with_snapshot` は action-string contract を使っており、full decision engine ではありません。
- episode は現在 lightweight projection であり、complete autobiographical memory model ではありません。
- provider live evidence は configuration と connectivity のみを証明します。model quality、SLA、production readiness は証明しません。
- local alpha gates は real fresh-machine run、Windows parity、human release decision などの external evidence に依存します。

Not implemented:

- Full memory layering
- Richer evidence ranking / weighting
- Production-grade remote, team, or multi-tenant mode
- Daemon write capabilities and autonomous background operation
- Installers, service managers, auto-updaters, and release certification

Complete implementation status is tracked in [project status](project-status.md) and [roadmap](roadmap.md).

## Documentation

- [Positioning](positioning.md)
- [FAQ](faq.md)
- [Project overview: English](project-overview.en.md)
- [Project overview: Simplified Chinese](project-overview.zh-CN.md)
- [Project overview: Japanese](project-overview.ja.md)
- [Testing guide](testing-guide-2026-03-24.md)
- [Release readiness](release-readiness.md)
- [Release gate runbook](release-gate.md)
- [Local Alpha PRD](product/prd-local-alpha.md)
- [Provider contract](provider-contract.md)
- [Document map](document-map.md)

## Verification

As of `2026-06-08`, `cargo test` passes with 400 tests.

Common local checks:

```zsh
cargo test
./scripts/agent-llm-mm.sh doctor
git diff --check
```

For provider, dashboard, release-evidence, or support-bundle changes, follow the relevant layered checks in the [testing guide](testing-guide-2026-03-24.md).

## Naming

The public project name is MCP Memory Ledger. The current Rust crate, binary, scripts, config examples, and some historical docs still use `agent_llm_mm` / `agent-llm-mm` as compatibility identifiers.

## Acknowledgements

This repository has been developed, reviewed, and documented with active support from OpenAI Codex as a collaborative development tool. Thanks to OpenAI for the tooling and research ecosystem that made this workflow possible.

## License

This project is licensed under the Apache License 2.0. See [LICENSE](../LICENSE) and [NOTICE](../NOTICE).

Copyright 2026 yooyui
