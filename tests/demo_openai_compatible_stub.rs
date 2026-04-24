use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn demo_stub_distinguishes_decision_and_self_revision_requests() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_demo_openai_compatible_stub"))
        .arg("--port")
        .arg("0")
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn demo stub");

    let stdout = child.stdout.take().expect("stub stdout");
    let mut startup_line = String::new();
    BufReader::new(stdout)
        .read_line(&mut startup_line)
        .expect("startup line");
    let startup: Value = serde_json::from_str(startup_line.trim()).expect("startup json");
    let base_url = startup["base_url"].as_str().expect("base_url");

    let client = reqwest::Client::new();

    let decision_response = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth("demo-test-key")
        .json(&json!({
            "model": "demo-local",
            "temperature": 0.0,
            "messages": [
                {
                    "role": "system",
                    "content": "Return only the next action name as plain text with no explanation."
                },
                {
                    "role": "user",
                    "content": "Task: review\nAction: review_conflicting_commitment_update\nSnapshot:\n{\n  \"identity\": [\"identity:self=architect\"],\n  \"commitments\": [],\n  \"claims\": [],\n  \"evidence\": [],\n  \"episodes\": []\n}"
                }
            ]
        }))
        .send()
        .await
        .expect("decision response")
        .json::<Value>()
        .await
        .expect("decision json");

    assert_eq!(
        decision_response["choices"][0]["message"]["content"],
        json!("apply_commitment_update_now")
    );

    let self_revision_response = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth("demo-test-key")
        .json(&json!({
            "model": "demo-local",
            "temperature": 0.0,
            "messages": [
                {
                    "role": "system",
                    "content": "Return only a JSON self-revision proposal with should_reflect, rationale, machine_patch.identity_patch, machine_patch.commitment_patch, proposed_evidence_event_ids, proposed_evidence_query, and confidence."
                },
                {
                    "role": "user",
                    "content": "Self revision request:\n{\n  \"trigger_type\": \"Conflict\",\n  \"namespace\": \"self\",\n  \"snapshot\": {\n    \"identity\": [\"identity:self=architect\"],\n    \"commitments\": [\"forbid:write_identity_core_directly\"],\n    \"claims\": [\"self:self.role is architect\"],\n    \"evidence\": [\"event:evt-1\"],\n    \"episodes\": [\"episode:demo-baseline\"]\n  },\n  \"evidence_event_ids\": [\"evt-1\"],\n  \"trigger_hints\": [\"conflict\", \"commitment\"]\n}"
                }
            ]
        }))
        .send()
        .await
        .expect("proposal response")
        .json::<Value>()
        .await
        .expect("proposal json");

    let proposal = self_revision_response["choices"][0]["message"]["content"]
        .as_str()
        .expect("proposal content");

    assert!(proposal.contains("\"should_reflect\":true"));
    assert!(proposal.contains("prefer:confirm_conflicting_commitment_updates_before_overwrite"));

    child.kill().expect("kill stub");
    child.wait().expect("wait stub");
}
