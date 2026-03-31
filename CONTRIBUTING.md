# Contributing

Thanks for your interest in improving `agent_llm_mm`.

## Before You Start

- Read [README.md](/D:/Code/agent_llm_mm/README.md)
- Read [当前实现状态](/D:/Code/agent_llm_mm/docs/project-status.md)
- Read [路线图](/D:/Code/agent_llm_mm/docs/roadmap.md)
- Read [测试指南](/D:/Code/agent_llm_mm/docs/testing-guide-2026-03-24.md)

This repository is a public technical demo and MVP. Please keep changes aligned with that positioning. Avoid turning incomplete experimental capabilities into product claims in docs or code comments.

## Environment

- Windows 11
- PowerShell 7
- Rust toolchain

Recommended shell:

```powershell
pwsh.exe
```

## Suggested Workflow

1. Open an issue or describe the scope clearly before making non-trivial changes.
2. Keep the blast radius small and focused.
3. Update docs when behavior, scope, or public-facing wording changes.
4. Prefer explicit wording for what is implemented, partially implemented, and not implemented.

## Verification

Run the following before asking for review:

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
pwsh -File .\scripts\agent-llm-mm.ps1 doctor
```

## License for Contributions

By intentionally submitting a contribution to this repository, you agree that it will be licensed under Apache License 2.0, consistent with the repository license.

## Documentation Expectations

If your change affects project positioning or collaboration, update the relevant docs:

- [README.md](/D:/Code/agent_llm_mm/README.md)
- [文档总览](/D:/Code/agent_llm_mm/docs/document-map.md)
- [当前实现状态](/D:/Code/agent_llm_mm/docs/project-status.md)
- [路线图](/D:/Code/agent_llm_mm/docs/roadmap.md)
- [发布准备评估](/D:/Code/agent_llm_mm/docs/release-readiness.md)

## Acknowledgement

This repository is developed with active support from OpenAI Codex during discussion, iteration, and documentation refinement. Thanks to OpenAI for making that workflow possible.
