use agent_llm_mm::{
    interfaces::dashboard::{
        DashboardRuntimeInfo, EventQuery, OperationEvent, OperationKind, OperationRecorder,
        OperationStatus, start_dashboard_service,
    },
    support::config::DashboardConfig,
};
use chrono::Utc;
use reqwest::Client;
use serde_json::json;

fn config(base_path: &str) -> DashboardConfig {
    DashboardConfig {
        enabled: true,
        host: "127.0.0.1".to_string(),
        port: 0,
        base_path: base_path.to_string(),
        event_capacity: 20,
        sse_enabled: true,
        open_browser: false,
        required: true,
    }
}

fn runtime() -> DashboardRuntimeInfo {
    DashboardRuntimeInfo {
        service_name: "agent-llm-mm".to_string(),
        transport: "stdio".to_string(),
        provider: "mock".to_string(),
        dashboard_enabled: true,
        read_only: true,
    }
}

fn recorder_with_event() -> OperationRecorder {
    let recorder = OperationRecorder::new(20);
    recorder.append(OperationEvent {
        id: "op_1".to_string(),
        sequence: 1,
        timestamp: Utc::now(),
        kind: OperationKind::Reflection,
        status: OperationStatus::Handled,
        operation: "auto_reflect".to_string(),
        namespace: Some("self".to_string()),
        summary: "proposal passed evidence gate".to_string(),
        correlation_id: None,
        payload: json!({ "reflection_id": "reflection-1" }),
    });
    recorder
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dashboard_serves_html_summary_events_detail_and_health() {
    let recorder = recorder_with_event();
    let handle = start_dashboard_service(config("/"), recorder.clone(), runtime())
        .await
        .expect("dashboard starts");
    let base_url = handle.base_url();
    let client = Client::new();

    let html = client
        .get(format!("{base_url}/"))
        .send()
        .await
        .expect("html response")
        .text()
        .await
        .expect("html body");
    assert!(html.contains("Memory-chan Live Desk"));
    assert!(html.contains("production dashboard"));

    let summary: serde_json::Value = client
        .get(format!("{base_url}/api/summary"))
        .send()
        .await
        .expect("summary response")
        .json()
        .await
        .expect("summary json");
    assert_eq!(summary["total_events"], 1);
    assert_eq!(summary["reflection_events"], 1);

    let events: serde_json::Value = client
        .get(format!("{base_url}/api/events?limit=5"))
        .send()
        .await
        .expect("events response")
        .json()
        .await
        .expect("events json");
    assert_eq!(events.as_array().expect("events array").len(), 1);

    let detail: serde_json::Value = client
        .get(format!("{base_url}/api/events/op_1"))
        .send()
        .await
        .expect("detail response")
        .json()
        .await
        .expect("detail json");
    assert_eq!(detail["id"], "op_1");
    assert_eq!(detail["read_only"], true);

    let health: serde_json::Value = client
        .get(format!("{base_url}/api/health"))
        .send()
        .await
        .expect("health response")
        .json()
        .await
        .expect("health json");
    assert_eq!(health["status"], "ok");
    assert_eq!(health["read_only"], true);

    drop(handle);
    let _ = recorder.recent(EventQuery::default());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dashboard_honors_configured_base_path() {
    let recorder = recorder_with_event();
    let handle = start_dashboard_service(config("/agent-llm-mm"), recorder, runtime())
        .await
        .expect("dashboard starts");
    let base_url = handle.base_url();

    let summary: serde_json::Value = Client::new()
        .get(format!("{base_url}/api/summary"))
        .send()
        .await
        .expect("summary response")
        .json()
        .await
        .expect("summary json");

    assert!(base_url.ends_with("/agent-llm-mm"));
    assert_eq!(summary["total_events"], 1);
}
