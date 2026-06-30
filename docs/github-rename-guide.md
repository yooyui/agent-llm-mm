# GitHub Rename Guide

Status on 2026-06-30: the GitHub repository has been renamed to `yooyui/mcp-memory-ledger`, and the local `origin` remote should point to `git@github.com:yooyui/mcp-memory-ledger.git`.

## Recommendation

Do not create a new repository. Rename and reposition the current repository so existing commits, release gates, documentation, and evidence history stay intact.

Use **MCP Memory Ledger** as the project name, while keeping `agent_llm_mm` as the current crate / binary / script / config compatibility name until a separate technical migration is planned.

Recommended first step:

```text
Display name: MCP Memory Ledger
Repository slug: mcp-memory-ledger
Current implementation package: agent_llm_mm
```

## Can The GitHub Repository Be Renamed?

Yes, a GitHub repository can be renamed from repository settings by an owner or admin. This can be done directly on GitHub when the owner is ready to make the remote change.

GitHub redirects many old repository URLs to the new location after a rename, including common clone / fetch / push access. Local clones can keep working through redirects, but the remote URL should still be updated to avoid confusion. GitHub does not redirect every integration case; for example, when a renamed repository is referenced as a GitHub Action in a workflow, consumers should update the action reference.

Package names, scripts, documentation links, badges, MCP config examples, external references, and local clone remotes may still need manual updates.

Because this project has scripts, examples, MCP config references, release docs, and historical docs using `agent-llm-mm` / `agent_llm_mm`, a repository rename should be treated as a migration, not a copy edit.

## Suggested Migration Order

1. Add the project name to README and docs.
2. Update GitHub About description and topics.
3. Add FAQ and positioning pages for search discoverability.
4. Keep old technical names working.
5. Rename the GitHub repository only after the public positioning is stable.
6. Update local remotes and external links after the rename.
7. Plan a separate crate / binary / script rename only if compatibility cost is acceptable.

## Suggested Post-Rename Local Remote Update

After the GitHub repository slug is changed, local clones should update the remote URL:

```bash
git remote set-url origin git@github.com:yooyui/mcp-memory-ledger.git
git remote -v
```

## Compatibility Note

Keep this compatibility sentence in public docs during the transition:

```text
MCP Memory Ledger uses `agent_llm_mm` as the current implementation package name for the Rust crate, scripts, and configuration examples.
```
