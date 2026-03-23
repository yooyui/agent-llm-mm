use std::{
    io::{self, BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde::Deserialize;
use serde_json::{Value, json};

#[tokio::test]
async fn server_exposes_expected_tools_over_stdio() {
    let client = test_support::spawn_stdio_client().await.unwrap();
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

mod test_support {
    use super::*;

    pub async fn spawn_stdio_client() -> io::Result<StdioClient> {
        StdioClient::spawn()
    }

    pub struct StdioClient {
        child: Child,
        stdin: ChildStdin,
        stdout: BufReader<ChildStdout>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Tool {
        pub name: String,
    }

    impl StdioClient {
        fn spawn() -> io::Result<Self> {
            let mut child = Command::new(env!("CARGO_BIN_EXE_agent_llm_mm"))
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
                child,
                stdin,
                stdout: BufReader::new(stdout),
            })
        }

        pub async fn list_all_tools(mut self) -> io::Result<Vec<Tool>> {
            self.initialize()?;
            self.list_tools()
        }

        fn initialize(&mut self) -> io::Result<()> {
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
            let mut line = String::new();
            let bytes_read = self.stdout.read_line(&mut line)?;
            if bytes_read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "child process closed stdout before sending an MCP message",
                ));
            }

            serde_json::from_str(&line)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        }
    }

    impl Drop for StdioClient {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
