//! Shared HTTP AppState (Analyze-84).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use aira_flow::LocalSession;
use aira_protocol::DiscoveryRegistry;

use super::auth::bearer_credential;
use crate::tenant_auth::{resolve_principal, Principal, TenantAuthMap};

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

    pub(super) fn principal_from_auth_header(&self, header: Option<&str>) -> Principal {
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
