//! Local HTTP API (Roadmap M11) — loopback JSON surface over LocalSession.
//!
//! Mechanical split (Analyze-84 / QUEUE #49).

mod auth;
mod handlers;
mod state;
mod util;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;

pub use state::AppState;

use auth::bearer_gate;
use handlers::{
    get_artifact, get_capabilities, get_csu_list, get_events, get_problem, get_result, health,
    post_conformance_run, post_csu_register, post_problem,
};

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

#[cfg(test)]
mod tests {
    use super::auth::bearer_credential;
    use super::*;
    use aira_flow::init_node;
    use aira_protocol::DiscoveryRegistry;
    use axum::http::header::AUTHORIZATION;
    use axum::http::StatusCode;
    use axum::Router;
    use http_body_util::BodyExt;
    use serde_json::{json, Value};
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
        assert_eq!(v["results"]["passed"], 8);
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
