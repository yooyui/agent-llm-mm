use std::{
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use agent_llm_mm::support::config::DATABASE_URL_ENV_VAR;
use serde::Deserialize;
use serde_json::{Value, json};
use tempfile::TempDir;

#[tokio::test]
async fn server_exposes_expected_tools_over_stdio() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let tools = client.list_all_tools().await.unwrap();
    let names = tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect::<Vec<_>>();

    assert!(names.contains(&"ingest_interaction".to_string()));
    assert!(names.contains(&"build_self_snapshot".to_string()));
    assert!(names.contains(&"decide_with_snapshot".to_string()));
    assert!(names.contains(&"run_reflection".to_string()));
}

#[tokio::test]
async fn stdio_tools_share_runtime_state_across_calls() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user asked for stronger memory."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:task-7"
            }),
        )
        .await
        .unwrap();
    let event_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .unwrap();
    assert!(!event_id.is_empty());

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let claims = snapshot
        .get("claims")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        claims.contains(&"self.role is architect"),
        "snapshot claims missing ingested claim: {claims:?}"
    );

    let evidence = snapshot
        .get("evidence")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(evidence.len(), 1, "expected one evidence reference");
    assert!(
        evidence[0].starts_with("event:"),
        "unexpected evidence reference: {:?}",
        evidence
    );

    let episodes = snapshot
        .get("episodes")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        episodes.contains(&"episode:task-7"),
        "snapshot episodes missing ingested episode: {episodes:?}"
    );
}

#[tokio::test]
async fn conflicting_reflection_over_stdio_removes_claim_from_active_snapshot() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    let ingest = client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Conversation",
                    "summary": "The user described a role conflict."
                },
                "claim_drafts": [
                    {
                        "owner": "Self_",
                        "subject": "self.role",
                        "predicate": "is",
                        "object": "architect",
                        "mode": "Observed"
                    }
                ],
                "episode_reference": "episode:task-8"
            }),
        )
        .await
        .unwrap();
    let event_id = ingest
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("event_id"))
        .and_then(Value::as_str)
        .unwrap()
        .to_string();

    let reflection = client
        .call_tool(
            "run_reflection",
            json!({
                "reflection": {
                    "summary": "This reflection conflicts with the previous claim."
                },
                "supersede_claim_id": format!("{event_id}:claim:0"),
                "replacement_claim": null
            }),
        )
        .await
        .unwrap();
    let replacement_claim_id = reflection
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("replacement_claim_id"));
    assert!(
        replacement_claim_id.is_some_and(Value::is_null),
        "conflicting reflection should not create a replacement claim: {reflection:?}"
    );

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let claims = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .and_then(|value| value.get("claims"))
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        !claims.contains(&"self.role is architect"),
        "conflicting reflection should remove disputed claims from active snapshot: {claims:?}"
    );
}

#[tokio::test]
async fn fresh_stdio_runtime_blocks_forbidden_action_with_seeded_commitment() {
    let mut client = test_support::spawn_stdio_client().await.unwrap();
    let _ = client.list_all_tools().await.unwrap();

    client
        .call_tool(
            "ingest_interaction",
            json!({
                "event": {
                    "owner": "User",
                    "kind": "Observation",
                    "summary": "Bootstrap one evidence event so the snapshot can be built."
                },
                "claim_drafts": [],
                "episode_reference": "episode:task-8-gate"
            }),
        )
        .await
        .unwrap();

    let snapshot = client
        .call_tool("build_self_snapshot", json!({ "budget": 4 }))
        .await
        .unwrap();
    let snapshot = snapshot
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("snapshot"))
        .cloned()
        .unwrap();

    let commitments = snapshot
        .get("commitments")
        .and_then(Value::as_array)
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        commitments.contains(&"forbid:write_identity_core_directly"),
        "fresh stdio runtime should seed the baseline commitment: {commitments:?}"
    );

    let decision = client
        .call_tool(
            "decide_with_snapshot",
            json!({
                "task": "attempt a forbidden direct identity write",
                "action": "write_identity_core_directly",
                "snapshot": snapshot,
            }),
        )
        .await
        .unwrap();

    let blocked = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_bool)
        .unwrap();
    let model_decision = decision
        .get("result")
        .and_then(|value| value.get("structuredContent"))
        .and_then(|value| value.get("decision"));

    assert!(blocked, "baseline commitment should block forbidden action");
    assert!(
        model_decision.is_some_and(Value::is_null),
        "blocked decisions should not call the model: {decision:?}"
    );
}

mod test_support {
    use super::*;

    pub async fn spawn_stdio_client() -> io::Result<StdioClient> {
        let database = database_override()?;
        StdioClient::spawn(&database.url, Some(database.temp_dir))
    }

    struct DatabaseOverride {
        temp_dir: TempDir,
        url: String,
    }

    pub struct StdioClient {
        _database_dir: Option<TempDir>,
        child: Child,
        initialized: bool,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Tool {
        pub name: String,
    }

    impl StdioClient {
        fn spawn(database_url: &str, database_dir: Option<TempDir>) -> io::Result<Self> {
            let mut child = Command::new(env!("CARGO_BIN_EXE_agent_llm_mm"))
                .env(DATABASE_URL_ENV_VAR, database_url)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| io::Error::other("missing child stdin"))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| io::Error::other("missing child stdout"))?;

            Ok(Self {
                _database_dir: database_dir,
                child,
                initialized: false,
                stdin,
                stdout: BufReader::new(stdout),
            })
        }

        pub async fn list_all_tools(&mut self) -> io::Result<Vec<Tool>> {
            self.initialize()?;
            self.list_tools()
        }

        pub async fn call_tool(&mut self, name: &str, arguments: Value) -> io::Result<Value> {
            self.initialize()?;
            self.send(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": name,
                    "arguments": arguments
                }
            }))?;
            self.read_message()
        }

        fn initialize(&mut self) -> io::Result<()> {
            if self.initialized {
                return Ok(());
            }

            self.send(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-03-26",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "mcp-stdio-test",
                        "version": "0.1.0"
                    }
                }
            }))?;
            let _ = self.read_message()?;

            self.send(json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }))?;
            self.initialized = true;
            Ok(())
        }

        fn list_tools(&mut self) -> io::Result<Vec<Tool>> {
            self.send(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }))?;

            let message = self.read_message()?;
            let tools = message
                .get("result")
                .and_then(|result| result.get("tools"))
                .cloned()
                .ok_or_else(|| io::Error::other("missing tools in response"))?;

            serde_json::from_value(tools)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        }

        fn send(&mut self, payload: Value) -> io::Result<()> {
            let mut body = serde_json::to_vec(&payload)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
            body.push(b'\n');
            self.stdin.write_all(&body)?;
            self.stdin.flush()
        }

        fn read_message(&mut self) -> io::Result<Value> {
            loop {
                let mut line = String::new();
                let bytes_read = self.stdout.read_line(&mut line)?;
                if bytes_read == 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "child process closed stdout before sending an MCP message",
                    ));
                }

                let trimmed = line.trim();
                if !trimmed.starts_with('{') {
                    continue;
                }

                return serde_json::from_str(trimmed)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
            }
        }
    }

    impl Drop for StdioClient {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn database_override() -> io::Result<DatabaseOverride> {
        let temp_dir = tempfile::tempdir()?;
        let database_path = temp_dir.path().join("agent-llm-mm.sqlite");
        Ok(DatabaseOverride {
            url: sqlite_url(&database_path),
            temp_dir,
        })
    }

    fn sqlite_url(path: &Path) -> String {
        format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"))
    }
}
