//! HTTP route handlers (Analyze-84). No new routes.

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use aira_conformance::{run_profile, ConformanceProfile};
use aira_csu::{CsuLifecycleState, CsuManifest, CsuRegistry};
use aira_flow::SubmitOutcome;

use super::state::AppState;
use super::util::{decode_id, err, hex_encode};
use crate::tenant_auth::{authorize_csu_register, filter_csu_list};

#[derive(serde::Serialize)]
pub(super) struct HealthBody {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

pub(super) async fn health() -> Json<HealthBody> {
    Json(HealthBody {
        status: "ok",
        service: "aira-node",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Deserialize)]
pub(super) struct ProblemSubmitBody {
    text: String,
}

pub(super) async fn post_problem(
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

pub(super) async fn get_problem(
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

pub(super) async fn get_result(
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

pub(super) async fn get_artifact(
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
pub(super) struct EventsQuery {
    #[serde(default = "default_event_limit")]
    limit: usize,
}

fn default_event_limit() -> usize {
    50
}

pub(super) async fn get_events(
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> Response {
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

pub(super) async fn get_capabilities(State(state): State<AppState>) -> Response {
    let discovery = match state.discovery.lock() {
        Ok(g) => g,
        Err(_) => return err(StatusCode::INTERNAL_SERVER_ERROR, "discovery lock poisoned"),
    };
    let caps: Vec<_> = discovery.list_all().into_iter().cloned().collect();
    Json(json!({ "capabilities": caps })).into_response()
}

pub(super) async fn get_csu_list(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Response {
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
pub(super) struct CsuRegisterBody {
    manifest: CsuManifest,
    #[serde(default)]
    activate: bool,
}

pub(super) async fn post_csu_register(
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
pub(super) struct ConformanceRunBody {
    profile: String,
}

pub(super) async fn post_conformance_run(
    State(state): State<AppState>,
    Json(body): Json<ConformanceRunBody>,
) -> Response {
    let profile = match body.profile.to_uppercase().as_str() {
        "C0" => ConformanceProfile::C0,
        "C1" => ConformanceProfile::C1,
        "C2" => ConformanceProfile::C2,
        "C3" => ConformanceProfile::C3,
        "C4" => ConformanceProfile::C4,
        other => {
            return err(
                StatusCode::BAD_REQUEST,
                &format!("unsupported profile {other} (use C0, C1, C2, C3, or C4)"),
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
