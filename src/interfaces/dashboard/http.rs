use std::{convert::Infallible, net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{
        Html, IntoResponse,
        sse::{Event, Sse},
    },
    routing::get,
};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};

use crate::support::config::DashboardConfig;

use super::{
    DashboardRuntimeInfo, EventQuery, OperationKind, OperationRecorder, OperationStatus,
    assets::{DASHBOARD_HTML, MEMORY_CHAN_HERO_PNG, MEMORY_CHAN_SIDEBAR_PNG},
    build_summary, project_event_detail,
};

#[derive(Clone)]
struct DashboardState {
    recorder: OperationRecorder,
    runtime: DashboardRuntimeInfo,
}

pub struct DashboardHandle {
    address: SocketAddr,
    base_path: String,
    task: tokio::task::JoinHandle<()>,
}

impl DashboardHandle {
    pub fn base_url(&self) -> String {
        if self.base_path == "/" {
            format!("http://{}", self.address)
        } else {
            format!("http://{}{}", self.address, self.base_path)
        }
    }
}

impl Drop for DashboardHandle {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Debug, Deserialize)]
struct EventsQuery {
    limit: Option<usize>,
    kind: Option<OperationKind>,
    status: Option<OperationStatus>,
    namespace: Option<String>,
}

pub async fn start_dashboard_service(
    config: DashboardConfig,
    recorder: OperationRecorder,
    runtime: DashboardRuntimeInfo,
) -> Result<DashboardHandle> {
    config.validate().map_err(anyhow::Error::msg)?;
    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    let address = listener.local_addr()?;
    let base_path = normalized_base_path(&config.base_path);
    let state = DashboardState { recorder, runtime };
    let app = router(state, &base_path, config.sse_enabled);
    let task = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            tracing::warn!(error = %error, "dashboard service stopped with error");
        }
    });

    Ok(DashboardHandle {
        address,
        base_path,
        task,
    })
}

fn router(state: DashboardState, base_path: &str, sse_enabled: bool) -> Router {
    let mut routes = Router::new()
        .route("/", get(index))
        .route("/assets/memory-chan-hero.png", get(memory_chan_hero))
        .route("/assets/memory-chan-sidebar.png", get(memory_chan_sidebar))
        .route("/api/summary", get(summary))
        .route("/api/events", get(events))
        .route("/api/events/{id}", get(event_detail))
        .route("/api/health", get(health));

    if sse_enabled {
        routes = routes.route("/api/events/stream", get(event_stream));
    }

    let routes = routes.with_state(Arc::new(state));
    if base_path == "/" {
        routes
    } else {
        Router::new().nest(base_path, routes)
    }
}

fn normalized_base_path(base_path: &str) -> String {
    let trimmed = base_path.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        trimmed.to_string()
    }
}

async fn index() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn memory_chan_hero() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/png")], MEMORY_CHAN_HERO_PNG)
}

async fn memory_chan_sidebar() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        MEMORY_CHAN_SIDEBAR_PNG,
    )
}

async fn summary(State(state): State<Arc<DashboardState>>) -> Json<super::DashboardSummary> {
    let events = state.recorder.recent(EventQuery::default());
    Json(build_summary(&events, &state.runtime))
}

async fn events(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<EventsQuery>,
) -> Json<Vec<super::OperationEvent>> {
    Json(state.recorder.recent(EventQuery {
        limit: query.limit,
        kind: query.kind,
        status: query.status,
        namespace: query.namespace,
    }))
}

async fn event_detail(
    State(state): State<Arc<DashboardState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let events = state.recorder.recent(EventQuery::default());
    match events.iter().find(|event| event.id == id) {
        Some(event) => Json(project_event_detail(event)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "operation event not found" })),
        )
            .into_response(),
    }
}

async fn event_stream(
    State(state): State<Arc<DashboardState>>,
) -> Sse<impl Stream<Item = std::result::Result<Event, Infallible>>> {
    let stream = BroadcastStream::new(state.recorder.subscribe()).filter_map(|event| match event {
        Ok(event) => Some(Ok(Event::default().json_data(event).unwrap_or_else(|_| {
            Event::default().data(r#"{"error":"failed to serialize dashboard event"}"#)
        }))),
        Err(_) => None,
    });
    Sse::new(stream)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "read_only": true }))
}
