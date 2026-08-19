//! HTTP Bearer gate (Analyze-48 / Analyze-84). Authz semantics unchanged.

use std::sync::Arc;

use axum::extract::Request;
use axum::http::header::AUTHORIZATION;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use super::util::err;
use crate::tenant_auth::{bearer_token_accepted, TenantAuthMap};

/// Reject unauthenticated `/v1/*` when a shared token is configured.
pub(super) async fn bearer_gate(
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
pub(super) fn bearer_credential(header: Option<&str>) -> Option<&str> {
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
