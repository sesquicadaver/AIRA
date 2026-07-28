//! Local HTTP API (Roadmap M11) — loopback JSON surface over LocalSession.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use aira_conformance::{run_profile, ConformanceProfile};
use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
use aira_flow::{LocalSession, SubmitOutcome};
use aira_protocol::DiscoveryRegistry;

/// Shared node state for HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub session: Arc<Mutex<LocalSession>>,
    pub discovery: Arc<Mutex<DiscoveryRegistry>>,
}

impl AppState {
    /// Open session and seed local capability descriptors from config autoload.
    ///
    /// Loads durable `.aira/discovery/registry.json` when present, seeds missing
    /// autoload capabilities, then persists.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, String> {
        let root = root.as_ref().to_path_buf();
        let session = LocalSession::open(&root).map_err(|e| e.to_string())?;
        let mut discovery = DiscoveryRegistry::load(&root).map_err(|e| e.to_string())?;
        for name in &session.config.csu.autoload {
            let cap_id = format!("aira:capability:local:{name}");
            if discovery.contains(&cap_id) {
                continue;
            }
            let csu_id = format!("aira:csu:{name}");
            let cap_type = format!("local.{name}");
            if let Ok(cap) = DiscoveryRegistry::local_capability(&cap_id, &cap_type, &csu_id) {
                let _ = discovery.register(cap);
            }
        }
        discovery.save(&root).map_err(|e| e.to_string())?;
        Ok(Self {
            root,
            session: Arc::new(Mutex::new(session)),
            discovery: Arc::new(Mutex::new(discovery)),
        })
    }
}

/// Build the M11 router (also used by integration tests).
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/problems", post(post_problem))
        .route("/v1/problems/:problem_id", get(get_problem))
        .route("/v1/results/:result_id", get(get_result))
        .route("/v1/artifacts/:artifact_id", get(get_artifact))
        .route("/v1/events", get(get_events))
        .route("/v1/capabilities", get(get_capabilities))
        .route("/v1/csu", get(get_csu_list))
        .route("/v1/csu/register", post(post_csu_register))
        .route("/v1/conformance/run", post(post_conformance_run))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthBody> {
    Json(HealthBody {
        status: "ok",
        service: "aira-node",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Deserialize)]
struct ProblemSubmitBody {
    text: String,
}

async fn post_problem(
    State(state): State<AppState>,
    Json(body): Json<ProblemSubmitBody>,
) -> Response {
    if body.text.trim().is_empty() {
        return err(StatusCode::BAD_REQUEST, "text must be non-empty");
    }
    let mut session = match state.session.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "session lock poisoned"),
    };
    match session.submit_problem(&body.text) {
        Ok(SubmitOutcome::Completed {
            problem_id,
            verified_artifact_id,
            result,
        }) => Json(json!({
            "status": "completed",
            "problem_id": problem_id.as_str(),
            "verified_artifact_id": verified_artifact_id.as_str(),
            "result": result,
        }))
        .into_response(),
        Ok(SubmitOutcome::NeedsHumanCollapse { field_artifact_id }) => {
            let problem_id = session
                .plane()
                .problem_ref()
                .map(|r| r.as_str().to_string())
                .unwrap_or_else(|| "aira:problem:unknown".into());
            Json(json!({
                "status": "needs_human_collapse",
                "problem_id": problem_id,
                "field_artifact_id": field_artifact_id.as_str(),
            }))
            .into_response()
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn get_problem(
    State(state): State<AppState>,
    AxumPath(problem_id): AxumPath<String>,
) -> Response {
    let id = decode_id(&problem_id);
    let session = match state.session.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "session lock poisoned"),
    };
    match session.problem_status(&id) {
        Ok(rec) => Json(rec).into_response(),
        Err(e) => err(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

async fn get_result(
    State(state): State<AppState>,
    AxumPath(result_id): AxumPath<String>,
) -> Response {
    let id = decode_id(&result_id);
    let session = match state.session.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "session lock poisoned"),
    };
    match session.get_result(&id) {
        Ok(v) => Json(v).into_response(),
        Err(e) => err(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

async fn get_artifact(
    State(state): State<AppState>,
    AxumPath(artifact_id): AxumPath<String>,
) -> Response {
    let id = decode_id(&artifact_id);
    let session = match state.session.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "session lock poisoned"),
    };
    match session.get_artifact(&id) {
        Ok((desc, bytes)) => {
            let mut out = serde_json::Map::new();
            out.insert("descriptor".into(), desc);
            if let Ok(body) = serde_json::from_slice::<Value>(&bytes) {
                out.insert("payload".into(), body);
            } else {
                out.insert("payload_hex".into(), Value::String(hex_encode(&bytes)));
            }
            Json(Value::Object(out)).into_response()
        }
        Err(e) => err(StatusCode::NOT_FOUND, &e.to_string()),
    }
}

#[derive(Deserialize)]
struct EventsQuery {
    #[serde(default = "default_event_limit")]
    limit: usize,
}

fn default_event_limit() -> usize {
    50
}

async fn get_events(State(state): State<AppState>, Query(q): Query<EventsQuery>) -> Response {
    let limit = q.limit.clamp(1, 10_000);
    let session = match state.session.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "session lock poisoned"),
    };
    match session.event_tail(limit) {
        Ok(events) => Json(json!({ "events": events, "limit": limit })).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

async fn get_capabilities(State(state): State<AppState>) -> Response {
    let discovery = match state.discovery.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "discovery lock poisoned"),
    };
    let caps: Vec<_> = discovery.list_all().into_iter().cloned().collect();
    Json(json!({ "capabilities": caps })).into_response()
}

async fn get_csu_list(State(state): State<AppState>) -> Response {
    let path = state.root.join("csu").join("registry.json");
    if !path.exists() {
        return Json(json!({ "csu": [] })).into_response();
    }
    match CsuRegistry::load(&path) {
        Ok(reg) => {
            let list: Vec<_> = reg
                .list()
                .into_iter()
                .map(|e| {
                    json!({
                        "csu_id": e.manifest.csu_id.as_str(),
                        "csu_name": e.manifest.csu_name,
                        "csu_type": format!("{:?}", e.manifest.csu_type),
                        "state": format!("{:?}", e.state),
                    })
                })
                .collect();
            Json(json!({ "csu": list })).into_response()
        }
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

#[derive(Deserialize)]
struct CsuRegisterBody {
    manifest: CsuManifest,
    #[serde(default)]
    activate: bool,
}

async fn post_csu_register(
    State(state): State<AppState>,
    Json(body): Json<CsuRegisterBody>,
) -> Response {
    let path = state.root.join("csu").join("registry.json");
    let mut reg = if path.exists() {
        match CsuRegistry::load(&path) {
            Ok(r) => r,
            Err(e) => return err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
        }
    } else {
        CsuRegistry::new()
    };
    let id = body.manifest.csu_id.clone();
    if let Err(e) = reg.register(body.manifest, None) {
        return err(StatusCode::BAD_REQUEST, &e.to_string());
    }
    if body.activate {
        if let Err(e) = reg.activate(&id, None) {
            return err(StatusCode::BAD_REQUEST, &e.to_string());
        }
    }
    if let Err(e) = reg.save(&path) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, &format!("save: {e}"));
    }
    let state_label = reg
        .get(&id)
        .map(|e| format!("{:?}", e.state))
        .unwrap_or_else(|| format!("{:?}", CsuLifecycleState::Registered));
    Json(json!({
        "csu_id": id.as_str(),
        "state": state_label,
        "registry": path.display().to_string(),
    }))
    .into_response()
}

#[derive(Deserialize)]
struct ConformanceRunBody {
    profile: String,
}

async fn post_conformance_run(
    State(state): State<AppState>,
    Json(body): Json<ConformanceRunBody>,
) -> Response {
    let profile = match body.profile.to_uppercase().as_str() {
        "C0" => ConformanceProfile::C0,
        "C1" => ConformanceProfile::C1,
        "C2" => ConformanceProfile::C2,
        other => {
            return err(
                StatusCode::BAD_REQUEST,
                &format!("unsupported profile {other} (use C0, C1, or C2)"),
            )
        }
    };
    let out = state.root.join("conformance").join("reports");
    if let Err(e) = std::fs::create_dir_all(&out) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
    }
    match run_profile(profile, &out) {
        Ok(suite) => Json(json!({
            "profile": suite.report.aira.profile.as_str(),
            "results": suite.report.results,
            "report_artifact_id": suite.report_artifact_id.as_str(),
            "failures": suite.report.failures,
        }))
        .into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

fn decode_id(raw: &str) -> String {
    // Axum may leave percent-encoding; also strip a leading slash from {*path}.
    let s = raw.trim_start_matches('/');
    match percent_decode(s) {
        Ok(d) => d,
        Err(_) => s.to_string(),
    }
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                let h = hex_nibble(bytes[i + 1])?;
                let l = hex_nibble(bytes[i + 2])?;
                out.push((h << 4) | l);
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| ())
}

fn hex_nibble(b: u8) -> Result<u8, ()> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(()),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}

fn err(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({ "error": message }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_flow::init_node;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn setup() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("node");
        init_node(&root).unwrap();
        let state = AppState::open(&root).unwrap();
        (dir, state)
    }

    async fn json_req(
        app: Router,
        method: &str,
        uri: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let builder = axum::http::Request::builder().method(method).uri(uri);
        let req = if let Some(b) = body {
            builder
                .header("content-type", "application/json")
                .body(axum::body::Body::from(serde_json::to_vec(&b).unwrap()))
                .unwrap()
        } else {
            builder.body(axum::body::Body::empty()).unwrap()
        };
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let v: Value = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        (status, v)
    }

    #[tokio::test]
    async fn http_health() {
        let (_dir, state) = setup();
        let (st, v) = json_req(router(state), "GET", "/health", None).await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn http_post_problem_2_plus_2() {
        let (_dir, state) = setup();
        let (st, v) = json_req(
            router(state),
            "POST",
            "/v1/problems",
            Some(json!({"text": "Calculate 2 + 2"})),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert_eq!(v["status"], "completed");
        assert!(v["problem_id"].as_str().unwrap().starts_with("aira:"));
        assert!(v.get("result").is_some());
    }

    #[tokio::test]
    async fn http_problem_status_roundtrip() {
        let (_dir, state) = setup();
        let app = router(state);
        let (st, v) = json_req(
            app.clone(),
            "POST",
            "/v1/problems",
            Some(json!({"text": "Calculate 2 + 2"})),
        )
        .await;
        assert_eq!(st, StatusCode::OK);
        let pid = v["problem_id"].as_str().unwrap().to_string();
        let (st2, rec) = json_req(app, "GET", &format!("/v1/problems/{pid}"), None).await;
        assert_eq!(st2, StatusCode::OK, "{rec}");
        assert_eq!(rec["problem_id"], pid);
        assert_eq!(rec["status"], "completed");
    }

    #[tokio::test]
    async fn http_get_result() {
        let (_dir, state) = setup();
        let app = router(state);
        let (st, v) = json_req(
            app.clone(),
            "POST",
            "/v1/problems",
            Some(json!({"text": "Calculate 2 + 2"})),
        )
        .await;
        assert_eq!(st, StatusCode::OK);
        let pid = v["problem_id"].as_str().unwrap();
        let (st2, res) = json_req(app, "GET", &format!("/v1/results/{pid}"), None).await;
        assert_eq!(st2, StatusCode::OK, "{res}");
        assert!(!res.is_null());
    }

    #[tokio::test]
    async fn http_get_artifact() {
        let (_dir, state) = setup();
        let app = router(state);
        let (st, v) = json_req(
            app.clone(),
            "POST",
            "/v1/problems",
            Some(json!({"text": "Calculate 2 + 2"})),
        )
        .await;
        assert_eq!(st, StatusCode::OK);
        let aid = v["verified_artifact_id"].as_str().unwrap();
        let (st2, art) = json_req(app, "GET", &format!("/v1/artifacts/{aid}"), None).await;
        assert_eq!(st2, StatusCode::OK, "{art}");
        assert!(art.get("descriptor").is_some());
    }

    #[tokio::test]
    async fn http_events_tail() {
        let (_dir, state) = setup();
        let app = router(state);
        let _ = json_req(
            app.clone(),
            "POST",
            "/v1/problems",
            Some(json!({"text": "Calculate 2 + 2"})),
        )
        .await;
        let (st, v) = json_req(app, "GET", "/v1/events?limit=10", None).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert!(!v["events"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn http_capabilities() {
        let (_dir, state) = setup();
        let root = state.root.clone();
        let (st, v) = json_req(router(state), "GET", "/v1/capabilities", None).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert!(!v["capabilities"].as_array().unwrap().is_empty());
        assert!(DiscoveryRegistry::path(&root).exists());
        let loaded = DiscoveryRegistry::load(&root).unwrap();
        assert!(!loaded.list_all().is_empty());
    }

    #[tokio::test]
    async fn http_csu_list() {
        let (_dir, state) = setup();
        let (st, v) = json_req(router(state), "GET", "/v1/csu", None).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert!(v["csu"].as_array().is_some());
    }

    #[tokio::test]
    async fn http_csu_register() {
        let (_dir, state) = setup();
        let manifest: Value =
            serde_json::from_str(include_str!("../../../fixtures/valid/csu/manifest.json"))
                .unwrap();
        let (st, v) = json_req(
            router(state),
            "POST",
            "/v1/csu/register",
            Some(json!({"manifest": manifest, "activate": true})),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert_eq!(v["csu_id"], "aira:csu:execution.basic");
    }

    #[tokio::test]
    async fn http_conformance_c2() {
        let (_dir, state) = setup();
        let (st, v) = json_req(
            router(state),
            "POST",
            "/v1/conformance/run",
            Some(json!({"profile": "C2"})),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert_eq!(v["profile"], "C2");
        assert_eq!(v["results"]["failed"], 0);
        assert_eq!(v["results"]["passed"], 5);
    }
}
