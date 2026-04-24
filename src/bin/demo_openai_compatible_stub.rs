use anyhow::{Context, Result};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    let port = parse_port(std::env::args().skip(1))?;
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    let base_url = format!("http://{}", listener.local_addr()?);

    println!("{}", json!({ "base_url": base_url }));

    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(error) = handle_connection(&mut stream).await {
                eprintln!("demo stub request failed: {error}");
            }
        });
    }
}

fn parse_port(mut args: impl Iterator<Item = String>) -> Result<u16> {
    match (args.next().as_deref(), args.next()) {
        (Some("--port"), Some(value)) => Ok(value.parse::<u16>()?),
        (None, None) => Ok(0),
        _ => anyhow::bail!("usage: demo_openai_compatible_stub [--port <port>]"),
    }
}

async fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let request = read_http_request(stream).await?;
    let body = request
        .split("\r\n\r\n")
        .nth(1)
        .context("missing http body")?;
    let json_body: Value = serde_json::from_str(body)?;
    let content = classify_response(&json_body);

    let response = json!({
        "id": "chatcmpl-demo",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            }
        }]
    });

    let response_text = response.to_string();
    let http = format!(
        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        response_text.len(),
        response_text
    );
    stream.write_all(http.as_bytes()).await?;
    Ok(())
}

async fn read_http_request(stream: &mut TcpStream) -> Result<String> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    let mut header_end = None;
    let mut content_length = None;

    loop {
        let bytes_read = stream.read(&mut chunk).await?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);

        if header_end.is_none() {
            header_end = find_header_end(&buffer);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                content_length = parse_content_length(&headers);
            }
        }

        if let (Some(end), Some(length)) = (header_end, content_length)
            && buffer.len() >= end + 4 + length
        {
            break;
        }
    }

    String::from_utf8(buffer).context("request was not valid utf-8")
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(headers: &str) -> Option<usize> {
    headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    })
}

fn classify_response(request: &Value) -> String {
    let system = request["messages"][0]["content"]
        .as_str()
        .unwrap_or_default();
    let user = request["messages"][1]["content"]
        .as_str()
        .unwrap_or_default();

    if system.contains("Return only the next action name") && user.contains("Task:") {
        let action =
            if user.contains("prefer:confirm_conflicting_commitment_updates_before_overwrite") {
                "confirm_conflicting_commitment_updates_before_overwrite"
            } else {
                "apply_commitment_update_now"
            };
        return action.to_string();
    }

    if system.contains("Return only a JSON self-revision proposal")
        && user.contains("Self revision request:")
    {
        return json!({
            "should_reflect": true,
            "rationale": "Conflict evidence suggests tighter commitment hygiene.",
            "machine_patch": {
                "identity_patch": null,
                "commitment_patch": {
                    "commitments": [
                        "prefer:confirm_conflicting_commitment_updates_before_overwrite"
                    ]
                }
            },
            "proposed_evidence_event_ids": [],
            "proposed_evidence_query": {
                "owner": "Self_",
                "kind": "Action",
                "limit": 1
            },
            "confidence": "high"
        })
        .to_string();
    }

    "unsupported_demo_request".to_string()
}
