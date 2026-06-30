# MCP Memory Ledger プロジェクト概要

## 要約

MCP Memory Ledger は、長期記憶・自己スナップショット・反省更新の最小ループを検証するための Rust 製ローカル MCP `stdio` memory demo です。現在の実装は SQLite を永続化基盤としており、完成した製品というより、技術デモ、統合プロトタイプ、研究向け MVP として位置付けるのが適切です。

互換性メモ：現在の Rust crate、binary、script、設定例、および一部の履歴ドキュメントでは、技術識別子として `agent_llm_mm` / `agent-llm-mm` がまだ使われています。公開プロジェクト名は MCP Memory Ledger に統一します。

## 現在のスコープ

- ローカル MCP `stdio` サーバー
- SQLite 永続化
- 設定ファイル駆動の provider 読み込み
- `openai-compatible` / OpenRouter provider
- `ingest_interaction`
- `build_self_snapshot`
- `decide_with_snapshot`
- `run_reflection`
- `doctor` / `serve` 起動エントリ

## 現在の境界

- `decide_with_snapshot` は `openai-compatible` または OpenRouter provider を利用できますが、返却契約はまだ最小の action string です
- リモート HTTP transport はありません
- richer evidence lookup / weight / relation は未実装です
- Azure とローカルモデル provider はまだ未実装です。OpenRouter は OpenAI-compatible transport を利用し、明示的な live runner は provider preflight evidence だけを生成します。provider quality、SLA、gateway certification ではありません
- release evidence index、provider certification preflight、packaging preflight、richer memory semantics projection はローカル read-only / preflight 能力です。不足している product evidence を生成せず、provider quality を認証せず、installer を作成せず、より完全な多層 memory モデルの完成も意味しません

## 適した用途

- ローカル AI クライアント統合の検証
- self-agent memory の技術デモ
- Rust + MCP + SQLite の最小構成リファレンス

## ドキュメント運用ルール

- 各タスクの完了後、そのタスクが挙動、能力境界、接続手順、設定、検証コマンド、または協業ルールに影響する場合は、対応するドキュメントを必ず同時に更新します。
- ドキュメント更新を最後にまとめて回すのではなく、可能な限りコード変更と同じタスク内で一緒に収束させます。

## 現在の検証状態

`2026-06-08` 時点で：

- `cargo test -- --list --format terse` は現在 395 tests を列挙します
- `doctor` は `status = ok` を返します

## 謝辞

このリポジトリは、OpenAI Codex を協調的な開発ツールとして活用しながら、実装・議論・文書整理を進めてきました。このようなワークフローを可能にするツール群と研究エコシステムを提供している OpenAI に感謝します。
