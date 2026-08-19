//! Local HTTP API (Roadmap M11) — loopback JSON surface over LocalSession.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use axum::extract::{Path as AxumPath, Query, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use aira_conformance::{run_profile, ConformanceProfile};
use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
use aira_flow::{LocalSession, SubmitOutcome};
use aira_protocol::DiscoveryRegistry;

use crate::tenant_auth::{
    authorize_csu_register, bearer_token_accepted, filter_csu_list, resolve_principal, Principal,
    TenantAuthMap,
};

/// Shared node state for HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub session: Arc<Mutex<LocalSession>>,
    pub discovery: Arc<Mutex<DiscoveryRegistry>>,
    /// When set, `/v1/*` requires `Authorization: Bearer <token>` (Analyze-48).
    pub http_token: Option<Arc<str>>,
    /// Optional tenant Bearer→publisher map (Analyze-64). Immutable for process lifetime.
    pub tenant_auth: Option<Arc<TenantAuthMap>>,
}

impl AppState {
    /// Open session and seed local capability descriptors from config autoload.
    ///
    /// Loads durable `.aira/discovery/registry.json` when present, seeds missing
    /// autoload capabilities, then persists. Bearer auth is off until
    /// [`Self::with_http_token`].
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
            http_token: None,
            tenant_auth: None,
        })
    }

    /// Enable optional shared-secret bearer auth (empty string clears).
    pub fn with_http_token(mut self, token: Option<String>) -> Self {
        self.http_token = token
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .map(Arc::from);
        self
    }

    /// Attach immutable tenant auth map (Analyze-64).
    pub fn with_tenant_auth(mut self, map: Option<TenantAuthMap>) -> Self {
        self.tenant_auth = map.map(Arc::new);
        self
    }

    fn principal_from_auth_header(&self, header: Option<&str>) -> Principal {
        if self.tenant_auth.is_none() {
            return Principal::Unscoped;
        }
        match bearer_credential(header) {
            Some(tok) => {
                resolve_principal(tok, self.http_token.as_deref(), self.tenant_auth.as_deref())
            }
            None => Principal::Unscoped,
        }
    }
}

/// Build the M11 router (also used by integration tests).
///
/// When `state.http_token` is set, all routes except `GET /health` require a
/// matching `Authorization: Bearer` header (admin token or tenant-map token).
pub fn router(state: AppState) -> Router {
    debug_assert!(
        state.http_token.is_some() || state.tenant_auth.is_none(),
        "tenant_auth map requires http_token (boot helper enforces)"
    );
    let expect_token = state.http_token.clone();
    let tenant_map = state.tenant_auth.clone();
    let app = Router::new()
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
        .with_state(state);

    match expect_token {
        Some(token) => app.layer(middleware::from_fn(move |req, next| {
            let token = token.clone();
            let map = tenant_map.clone();
            async move { bearer_gate(req, next, token, map).await }
        })),
        None => app,
    }
}

/// Plain-HTTP probe router: **only** `GET /health` (Analyze-56).
///
/// Used on `--health-listen` so liveness does not require an mTLS client cert.
pub fn health_router() -> Router {
    Router::new().route("/health", get(health))
}

/// Reject unauthenticated `/v1/*` when a shared token is configured.
async fn bearer_gate(
    req: Request,
    next: Next,
    admin: Arc<str>,
    map: Option<Arc<TenantAuthMap>>,
) -> Response {
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }
    match bearer_credential(
        req.headers()
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok()),
    ) {
        Some(got) if bearer_token_accepted(got, Some(admin.as_ref()), map.as_deref()) => {
            next.run(req).await
        }
        _ => err(StatusCode::UNAUTHORIZED, "unauthorized"),
    }
}

/// Parse `Bearer <token>` (case-insensitive scheme).
fn bearer_credential(header: Option<&str>) -> Option<&str> {
    let raw = header?.trim();
    let (scheme, rest) = raw.split_once(char::is_whitespace)?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    let token = rest.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
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

async fn get_csu_list(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    let principal =
        state.principal_from_auth_header(headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()));
    let path = state.root.join("csu").join("registry.json");
    if !path.exists() {
        return Json(json!({ "csu": [] })).into_response();
    }
    match CsuRegistry::load(&path) {
        Ok(reg) => {
            let all = reg.list();
            let filtered =
                filter_csu_list(&principal, &all, |e| e.manifest.publisher_identity.as_str());
            let list: Vec<_> = filtered
                .into_iter()
                .map(|e| {
                    json!({
                        "csu_id": e.manifest.csu_id.as_str(),
                        "csu_name": e.manifest.csu_name,
                        "csu_type": format!("{:?}", e.manifest.csu_type),
                        "publisher_identity": e.manifest.publisher_identity.as_str(),
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
    headers: axum::http::HeaderMap,
    Json(body): Json<CsuRegisterBody>,
) -> Response {
    let principal =
        state.principal_from_auth_header(headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()));
    if let Err(e) = authorize_csu_register(&principal, &body.manifest) {
        return err(StatusCode::FORBIDDEN, &e.message);
    }
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
        json_req_auth(app, method, uri, body, None).await
    }

    async fn json_req_auth(
        app: Router,
        method: &str,
        uri: &str,
        body: Option<Value>,
        bearer: Option<&str>,
    ) -> (StatusCode, Value) {
        let mut builder = axum::http::Request::builder().method(method).uri(uri);
        if let Some(tok) = bearer {
            builder = builder.header(AUTHORIZATION, format!("Bearer {tok}"));
        }
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
        use aira_csu::support::basic_manifest;
        use aira_csu::CsuType;

        let (_dir, state) = setup();
        let manifest = basic_manifest(
            "aira:csu:execution.basic",
            "execution-basic",
            CsuType::Execution,
            &["CapsuleCreated"],
            &["CapsuleCompleted"],
        );
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

    #[tokio::test]
    async fn http_bearer_rejects_without_token() {
        let (_dir, state) = setup();
        let app = router(state.with_http_token(Some("secret-token".into())));
        let (st, v) = json_req(app, "GET", "/v1/capabilities", None).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{v}");
        assert_eq!(v["error"], "unauthorized");
    }

    #[tokio::test]
    async fn http_bearer_rejects_wrong_token() {
        let (_dir, state) = setup();
        let app = router(state.with_http_token(Some("secret-token".into())));
        let (st, v) = json_req_auth(app, "GET", "/v1/capabilities", None, Some("wrong")).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{v}");
    }

    #[tokio::test]
    async fn http_bearer_allows_with_token() {
        let (_dir, state) = setup();
        let app = router(state.with_http_token(Some("secret-token".into())));
        let (st, v) =
            json_req_auth(app, "GET", "/v1/capabilities", None, Some("secret-token")).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert!(v["capabilities"].as_array().is_some());
    }

    #[tokio::test]
    async fn http_bearer_health_exempt() {
        let (_dir, state) = setup();
        let app = router(state.with_http_token(Some("secret-token".into())));
        let (st, v) = json_req(app, "GET", "/health", None).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert_eq!(v["status"], "ok");
    }

    /// Bearer remains HTTP-layer independent of mTLS (Analyze-51 coexistence).
    #[tokio::test]
    async fn http_bearer_still_enforced_alongside_mtls_config() {
        let (_dir, state) = setup();
        // mTLS is transport-only; AppState bearer gate must still 401 without token.
        let app = router(state.with_http_token(Some("mtls-and-bearer".into())));
        let (st, v) = json_req(app.clone(), "GET", "/v1/capabilities", None).await;
        assert_eq!(st, StatusCode::UNAUTHORIZED, "{v}");
        let (st2, v2) = json_req_auth(
            app,
            "GET",
            "/v1/capabilities",
            None,
            Some("mtls-and-bearer"),
        )
        .await;
        assert_eq!(st2, StatusCode::OK, "{v2}");
    }

    #[tokio::test]
    async fn health_router_only_health() {
        let (st, v) = json_req(health_router(), "GET", "/health", None).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        assert_eq!(v["status"], "ok");
        let (st2, _) = json_req(health_router(), "GET", "/v1/capabilities", None).await;
        assert_eq!(st2, StatusCode::NOT_FOUND);
    }

    #[test]
    fn bearer_credential_parses_case_insensitive() {
        assert_eq!(bearer_credential(Some("Bearer abc")), Some("abc"));
        assert_eq!(bearer_credential(Some("bearer abc")), Some("abc"));
        assert_eq!(bearer_credential(Some("Basic abc")), None);
        assert_eq!(bearer_credential(Some("Bearer")), None);
        assert_eq!(bearer_credential(None), None);
    }

    #[test]
    fn constant_time_eq_basic() {
        use crate::tenant_auth::constant_time_eq;
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
    }

    #[tokio::test]
    async fn tenant_register_ok_and_cross_forbidden() {
        use crate::tenant_auth::{save_tenant_auth_map, TenantAuthEntry, TenantAuthMap};
        use aira_csu::support::{apply_publisher, basic_manifest};
        use aira_csu::CsuType;
        use aira_object::AiraRef;

        let (_dir, state) = setup();
        let map_path = state.root.join("identity").join("http-tenant-auth.json");
        save_tenant_auth_map(
            &map_path,
            &TenantAuthMap {
                version: 1,
                entries: vec![TenantAuthEntry {
                    token: "tenant-tok".into(),
                    publisher_id: "aira:identity:tenant-pub".into(),
                }],
            },
        )
        .unwrap();
        let map = crate::tenant_auth::load_tenant_auth_map(&map_path).unwrap();
        let app = router(
            state
                .with_http_token(Some("admin-tok".into()))
                .with_tenant_auth(Some(map)),
        );

        let mut ok_manifest = basic_manifest(
            "aira:csu:tenant.worker",
            "tenant-worker",
            CsuType::Execution,
            &[],
            &[],
        );
        apply_publisher(
            &mut ok_manifest,
            AiraRef::parse("aira:identity:tenant-pub").unwrap(),
        );
        let (st, v) = json_req_auth(
            app.clone(),
            "POST",
            "/v1/csu/register",
            Some(json!({"manifest": ok_manifest, "activate": false})),
            Some("tenant-tok"),
        )
        .await;
        assert_eq!(st, StatusCode::OK, "{v}");

        let mut bad = basic_manifest(
            "aira:csu:other.worker",
            "other-worker",
            CsuType::Execution,
            &[],
            &[],
        );
        apply_publisher(&mut bad, AiraRef::parse("aira:identity:other-pub").unwrap());
        let (st2, v2) = json_req_auth(
            app,
            "POST",
            "/v1/csu/register",
            Some(json!({"manifest": bad, "activate": false})),
            Some("tenant-tok"),
        )
        .await;
        assert_eq!(st2, StatusCode::FORBIDDEN, "{v2}");
        assert!(v2["error"].as_str().unwrap().contains("forbidden"));
    }

    #[tokio::test]
    async fn tenant_list_filtered_admin_sees_all() {
        use crate::tenant_auth::{save_tenant_auth_map, TenantAuthEntry, TenantAuthMap};
        use aira_csu::support::{apply_publisher, basic_manifest};
        use aira_csu::{CsuRegistry, CsuType};
        use aira_object::AiraRef;

        let (_dir, state) = setup();
        let reg_path = state.root.join("csu").join("registry.json");
        let mut reg = CsuRegistry::new();
        let mut m1 = basic_manifest("aira:csu:t1", "t1", CsuType::Execution, &[], &[]);
        apply_publisher(&mut m1, AiraRef::parse("aira:identity:tenant-pub").unwrap());
        let mut m2 = basic_manifest("aira:csu:t2", "t2", CsuType::Execution, &[], &[]);
        apply_publisher(&mut m2, AiraRef::parse("aira:identity:other-pub").unwrap());
        reg.register(m1, None).unwrap();
        reg.register(m2, None).unwrap();
        reg.save(&reg_path).unwrap();

        let map_path = state.root.join("identity").join("http-tenant-auth.json");
        save_tenant_auth_map(
            &map_path,
            &TenantAuthMap {
                version: 1,
                entries: vec![TenantAuthEntry {
                    token: "tenant-tok".into(),
                    publisher_id: "aira:identity:tenant-pub".into(),
                }],
            },
        )
        .unwrap();
        let map = crate::tenant_auth::load_tenant_auth_map(&map_path).unwrap();
        let app = router(
            state
                .with_http_token(Some("admin-tok".into()))
                .with_tenant_auth(Some(map)),
        );

        let (st, v) = json_req_auth(app.clone(), "GET", "/v1/csu", None, Some("tenant-tok")).await;
        assert_eq!(st, StatusCode::OK, "{v}");
        let arr = v["csu"].as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["csu_id"], "aira:csu:t1");

        let (st2, v2) = json_req_auth(app, "GET", "/v1/csu", None, Some("admin-tok")).await;
        assert_eq!(st2, StatusCode::OK, "{v2}");
        assert_eq!(v2["csu"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn legacy_no_map_unscoped_ok() {
        let (_dir, state) = setup();
        let app = router(state.with_http_token(Some("only-admin".into())));
        let (st, v) = json_req_auth(app, "GET", "/v1/csu", None, Some("only-admin")).await;
        assert_eq!(st, StatusCode::OK, "{v}");
    }

    #[tokio::test]
    async fn tenant_map_token_accepted_without_admin_secret_match() {
        use crate::tenant_auth::{save_tenant_auth_map, TenantAuthEntry, TenantAuthMap};

        let (_dir, state) = setup();
        let map_path = state.root.join("identity").join("http-tenant-auth.json");
        save_tenant_auth_map(
            &map_path,
            &TenantAuthMap {
                version: 1,
                entries: vec![TenantAuthEntry {
                    token: "tenant-only".into(),
                    publisher_id: "aira:identity:tenant-pub".into(),
                }],
            },
        )
        .unwrap();
        let map = crate::tenant_auth::load_tenant_auth_map(&map_path).unwrap();
        let app = router(
            state
                .with_http_token(Some("admin-tok".into()))
                .with_tenant_auth(Some(map)),
        );
        let (st, v) =
            json_req_auth(app, "GET", "/v1/capabilities", None, Some("tenant-only")).await;
        assert_eq!(st, StatusCode::OK, "{v}");
    }
}
