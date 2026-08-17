//! Multi-tenant HTTP authz for CSU routes (Analyze-64 / QUEUE #29).
//!
//! Bearer token map → `publisher_id`. mTLS CN→principal is out of this row.

use std::fs;
use std::path::{Path, PathBuf};

use aira_csu::CsuManifest;
use aira_object::AiraRef;
use serde::{Deserialize, Serialize};

/// Relative default path under node root.
pub const HTTP_TENANT_AUTH_FILE: &str = "http-tenant-auth.json";

/// Default map path: `<root>/identity/http-tenant-auth.json`.
pub fn default_tenant_auth_path(root: impl AsRef<Path>) -> PathBuf {
    root.as_ref().join("identity").join(HTTP_TENANT_AUTH_FILE)
}

/// Authenticated principal after Bearer authn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Principal {
    /// No tenant map loaded — legacy full access (after optional Bearer).
    Unscoped,
    /// `--http-token` matched and not present in the tenant map.
    Admin,
    /// Map entry matched (wins over admin when same secret).
    Tenant { publisher_id: String },
}

/// One Bearer → publisher binding (secret never logged).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantAuthEntry {
    pub token: String,
    pub publisher_id: String,
}

/// Durable tenant auth map (`identity/http-tenant-auth.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TenantAuthMap {
    pub version: u32,
    pub entries: Vec<TenantAuthEntry>,
}

impl TenantAuthMap {
    /// Validate structural invariants (no I/O).
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!(
                "http-tenant-auth.json unsupported version {}",
                self.version
            ));
        }
        let mut seen: Vec<&str> = Vec::new();
        for (i, e) in self.entries.iter().enumerate() {
            let tok = e.token.trim();
            let pub_id = e.publisher_id.trim();
            if tok.is_empty() {
                return Err(format!("http-tenant-auth.json entries[{i}]: empty token"));
            }
            if pub_id.is_empty() {
                return Err(format!(
                    "http-tenant-auth.json entries[{i}]: empty publisher_id"
                ));
            }
            AiraRef::parse(pub_id).map_err(|err| {
                format!("http-tenant-auth.json entries[{i}]: invalid publisher_id: {err}")
            })?;
            for prev in &seen {
                if constant_time_eq(prev.as_bytes(), tok.as_bytes()) {
                    return Err(format!(
                        "http-tenant-auth.json duplicate token at entries[{i}]"
                    ));
                }
            }
            seen.push(tok);
        }
        Ok(())
    }
}

/// Authz failure (map to HTTP 403).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthzError {
    pub message: String,
}

impl AuthzError {
    fn forbidden(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// Constant-time equality for equal-length secrets (no early exit on mismatch bytes).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Load map from disk. Rejects world/group-readable modes on unix.
pub fn load_tenant_auth_map(path: impl AsRef<Path>) -> Result<TenantAuthMap, String> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(format!(
            "http-tenant-auth map not found: {}",
            path.display()
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path)
            .map_err(|e| e.to_string())?
            .permissions()
            .mode()
            & 0o777;
        if mode & 0o077 != 0 {
            return Err(format!(
                "http-tenant-auth.json mode {mode:04o} too open (need 0600): {}",
                path.display()
            ));
        }
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let map: TenantAuthMap = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    map.validate()?;
    Ok(map)
}

/// Write map with mode `0600` (tests / operator tooling).
#[cfg(test)]
pub fn save_tenant_auth_map(path: impl AsRef<Path>, map: &TenantAuthMap) -> Result<(), String> {
    map.validate()?;
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let out = serde_json::to_string_pretty(map).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut opts = fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true).mode(0o600);
        let mut f = opts.open(path).map_err(|e| e.to_string())?;
        f.write_all(format!("{out}\n").as_bytes())
            .map_err(|e| e.to_string())?;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        fs::write(path, format!("{out}\n")).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Boot: load map if present / required; fail if map without Bearer token.
pub fn validate_http_auth_boot(
    http_token: Option<&str>,
    map_path: &Path,
    explicit_path: bool,
) -> Result<Option<TenantAuthMap>, String> {
    let token_set = http_token.map(|t| !t.trim().is_empty()).unwrap_or(false);
    if explicit_path {
        if !map_path.is_file() {
            return Err(format!(
                "--http-tenant-auth file not found: {}",
                map_path.display()
            ));
        }
        if !token_set {
            return Err(
                "http-tenant-auth map requires --http-token / AIRA_HTTP_TOKEN (fail closed)".into(),
            );
        }
        return Ok(Some(load_tenant_auth_map(map_path)?));
    }
    if map_path.is_file() {
        if !token_set {
            return Err(format!(
                "found {} but --http-token unset — refuse boot (fail closed)",
                map_path.display()
            ));
        }
        return Ok(Some(load_tenant_auth_map(map_path)?));
    }
    Ok(None)
}

/// Whether `got` matches admin token and/or any map entry (full scan, no early exit).
pub fn bearer_token_accepted(got: &str, admin: Option<&str>, map: Option<&TenantAuthMap>) -> bool {
    let mut ok = false;
    if let Some(a) = admin {
        ok |= constant_time_eq(got.as_bytes(), a.as_bytes());
    }
    if let Some(m) = map {
        for e in &m.entries {
            ok |= constant_time_eq(got.as_bytes(), e.token.trim().as_bytes());
        }
    }
    ok
}

/// Resolve principal after successful Bearer authn.
///
/// Map match wins over admin when the same secret appears in both.
pub fn resolve_principal(got: &str, admin: Option<&str>, map: Option<&TenantAuthMap>) -> Principal {
    let Some(m) = map else {
        return Principal::Unscoped;
    };
    let mut matched_pub: Option<String> = None;
    let mut found = false;
    for e in &m.entries {
        let eq = constant_time_eq(got.as_bytes(), e.token.trim().as_bytes());
        // Always evaluate `eq` (full scan); record first match only.
        if eq && !found {
            matched_pub = Some(e.publisher_id.trim().to_string());
            found = true;
        }
    }
    if let Some(publisher_id) = matched_pub {
        return Principal::Tenant { publisher_id };
    }
    if let Some(a) = admin {
        if constant_time_eq(got.as_bytes(), a.as_bytes()) {
            return Principal::Admin;
        }
    }
    // Authenticated path should not reach here; treat as unscoped fail-soft.
    Principal::Unscoped
}

/// Authorize CSU register against principal.
pub fn authorize_csu_register(
    principal: &Principal,
    manifest: &CsuManifest,
) -> Result<(), AuthzError> {
    match principal {
        Principal::Unscoped | Principal::Admin => Ok(()),
        Principal::Tenant { publisher_id } => {
            if manifest.publisher_identity.as_str() == publisher_id.as_str() {
                Ok(())
            } else {
                Err(AuthzError::forbidden(format!(
                    "forbidden: publisher_identity {} not allowed for this tenant (expected {publisher_id})",
                    manifest.publisher_identity.as_str()
                )))
            }
        }
    }
}

/// Filter CSU list for tenant principals.
pub fn filter_csu_list<'a, T, F>(
    principal: &Principal,
    entries: &'a [T],
    publisher_of: F,
) -> Vec<&'a T>
where
    F: Fn(&T) -> &str,
{
    match principal {
        Principal::Unscoped | Principal::Admin => entries.iter().collect(),
        Principal::Tenant { publisher_id } => entries
            .iter()
            .filter(|e| publisher_of(e) == publisher_id.as_str())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_csu::support::basic_manifest;
    use aira_csu::CsuType;
    use tempfile::tempdir;

    #[test]
    fn duplicate_token_in_map_rejects_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("m.json");
        let map = TenantAuthMap {
            version: 1,
            entries: vec![
                TenantAuthEntry {
                    token: "same".into(),
                    publisher_id: "aira:identity:a".into(),
                },
                TenantAuthEntry {
                    token: "same".into(),
                    publisher_id: "aira:identity:b".into(),
                },
            ],
        };
        fs::write(&path, serde_json::to_string(&map).unwrap()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        let err = load_tenant_auth_map(&path).unwrap_err();
        assert!(err.contains("duplicate"), "{err}");
    }

    #[test]
    fn empty_token_or_publisher_rejects_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("m.json");
        fs::write(
            &path,
            r#"{"version":1,"entries":[{"token":"","publisher_id":"aira:identity:a"}]}"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert!(load_tenant_auth_map(&path)
            .unwrap_err()
            .contains("empty token"));
        fs::write(
            &path,
            r#"{"version":1,"entries":[{"token":"t","publisher_id":""}]}"#,
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        assert!(load_tenant_auth_map(&path)
            .unwrap_err()
            .contains("empty publisher"));
    }

    #[cfg(unix)]
    #[test]
    fn map_mode_rejects_world_readable() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("m.json");
        let map = TenantAuthMap {
            version: 1,
            entries: vec![TenantAuthEntry {
                token: "t".into(),
                publisher_id: "aira:identity:a".into(),
            }],
        };
        save_tenant_auth_map(&path, &map).unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        let err = load_tenant_auth_map(&path).unwrap_err();
        assert!(err.contains("too open"), "{err}");
    }

    #[test]
    fn map_token_wins_over_admin_same_secret() {
        let map = TenantAuthMap {
            version: 1,
            entries: vec![TenantAuthEntry {
                token: "shared".into(),
                publisher_id: "aira:identity:tenant-pub".into(),
            }],
        };
        let p = resolve_principal("shared", Some("shared"), Some(&map));
        assert_eq!(
            p,
            Principal::Tenant {
                publisher_id: "aira:identity:tenant-pub".into()
            }
        );
    }

    #[test]
    fn admin_when_not_in_map() {
        let map = TenantAuthMap {
            version: 1,
            entries: vec![TenantAuthEntry {
                token: "tenant".into(),
                publisher_id: "aira:identity:t".into(),
            }],
        };
        assert_eq!(
            resolve_principal("admin", Some("admin"), Some(&map)),
            Principal::Admin
        );
    }

    #[test]
    fn unscoped_without_map() {
        assert_eq!(resolve_principal("x", Some("x"), None), Principal::Unscoped);
    }

    #[test]
    fn authorize_and_filter() {
        let mut manifest =
            basic_manifest("aira:csu:worker", "worker", CsuType::Execution, &[], &[]);
        manifest.publisher_identity = AiraRef::parse("aira:identity:tenant-pub").unwrap();
        let tenant = Principal::Tenant {
            publisher_id: "aira:identity:tenant-pub".into(),
        };
        authorize_csu_register(&tenant, &manifest).unwrap();
        manifest.publisher_identity = AiraRef::parse("aira:identity:other").unwrap();
        assert!(authorize_csu_register(&tenant, &manifest).is_err());

        #[derive(Clone)]
        struct E {
            pub_id: String,
        }
        let entries = [
            E {
                pub_id: "aira:identity:tenant-pub".into(),
            },
            E {
                pub_id: "aira:identity:other".into(),
            },
        ];
        let filtered = filter_csu_list(&tenant, &entries, |e| e.pub_id.as_str());
        assert_eq!(filtered.len(), 1);
        assert_eq!(
            filter_csu_list(&Principal::Admin, &entries, |e| e.pub_id.as_str()).len(),
            2
        );
    }

    #[test]
    fn map_without_http_token_boot_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("m.json");
        save_tenant_auth_map(
            &path,
            &TenantAuthMap {
                version: 1,
                entries: vec![TenantAuthEntry {
                    token: "t".into(),
                    publisher_id: "aira:identity:a".into(),
                }],
            },
        )
        .unwrap();
        let err = validate_http_auth_boot(None, &path, false).unwrap_err();
        assert!(
            err.contains("fail closed") || err.contains("http-token"),
            "{err}"
        );
        let err2 = validate_http_auth_boot(None, &path, true).unwrap_err();
        assert!(err2.contains("http-token"), "{err2}");
    }

    #[test]
    fn explicit_tenant_auth_path_missing_boot_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("missing.json");
        let err = validate_http_auth_boot(Some("tok"), &path, true).unwrap_err();
        assert!(err.contains("not found"), "{err}");
        assert!(validate_http_auth_boot(Some("tok"), &path, false)
            .unwrap()
            .is_none());
    }

    #[test]
    fn bearer_accepted_scans_all() {
        let map = TenantAuthMap {
            version: 1,
            entries: vec![
                TenantAuthEntry {
                    token: "a".into(),
                    publisher_id: "aira:identity:a".into(),
                },
                TenantAuthEntry {
                    token: "b".into(),
                    publisher_id: "aira:identity:b".into(),
                },
            ],
        };
        assert!(bearer_token_accepted("b", Some("admin"), Some(&map)));
        assert!(bearer_token_accepted("admin", Some("admin"), Some(&map)));
        assert!(!bearer_token_accepted("nope", Some("admin"), Some(&map)));
    }
}
