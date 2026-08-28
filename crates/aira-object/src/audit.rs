//! Append-only trust / ceremony audit log (Analyze-40).
//!
//! Path: `<root>/identity/trust-audit.jsonl` — one JSON object per line.
//! Never records signing secrets (public key hex only).

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::crypto::{utc_now_rfc3339, CryptoError};

/// Relative file name under `identity/`.
pub const TRUST_AUDIT_FILE: &str = "trust-audit.jsonl";

/// Ceremony / CRL action recorded in the audit log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustAuditAction {
    Revoke,
    Unrevoke,
    Rotate,
    Rekey,
    NodeRotate,
    /// Per-CSU tenant signing secret rotated (same publisher_id).
    TenantRotate,
    /// Per-CSU tenant signing secret revoked (dir removed).
    TenantRevoke,
    /// Local federation membership established (descriptor ceremony).
    FederationJoin,
    /// Local federation membership cleared (`federation leave`).
    FederationLeave,
    /// Cross-federation export denied by local IO policy.
    FederationExportDeny,
    /// Cross-federation import denied by local IO policy.
    FederationImportDeny,
}

impl TrustAuditAction {
    /// Stable string for CLI / display.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Revoke => "revoke",
            Self::Unrevoke => "unrevoke",
            Self::Rotate => "rotate",
            Self::Rekey => "rekey",
            Self::NodeRotate => "node_rotate",
            Self::TenantRotate => "tenant_rotate",
            Self::TenantRevoke => "tenant_revoke",
            Self::FederationJoin => "federation_join",
            Self::FederationLeave => "federation_leave",
            Self::FederationExportDeny => "federation_export_deny",
            Self::FederationImportDeny => "federation_import_deny",
        }
    }
}

/// One durable audit record (no secrets).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustAuditEntry {
    pub recorded_at: String,
    pub action: TrustAuditAction,
    pub subject_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grace_until: Option<String>,
    /// `cli` | `peer-delta` | `node-rotate` | `csu-tenant` (extensible).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Authenticated issuer when `source` is peer-delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer_id: Option<String>,
}

impl TrustAuditEntry {
    /// Build an entry stamped with current UTC (RFC3339).
    pub fn new(
        action: TrustAuditAction,
        subject_id: impl Into<String>,
        source: Option<&str>,
    ) -> Result<Self, CryptoError> {
        Ok(Self {
            recorded_at: utc_now_rfc3339()?,
            action,
            subject_id: subject_id.into(),
            new_id: None,
            reason: None,
            public_key_hex: None,
            grace_until: None,
            source: source
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            issuer_id: None,
        })
    }

    /// Attach optional reason (empty → none).
    pub fn with_reason(mut self, reason: Option<&str>) -> Self {
        self.reason = reason
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }

    /// Attach successor identity (rotate).
    pub fn with_new_id(mut self, new_id: Option<&str>) -> Self {
        self.new_id = new_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }

    /// Attach public key hex (never a secret).
    pub fn with_pubkey_hex(mut self, pubkey_hex: Option<&str>) -> Self {
        self.public_key_hex = pubkey_hex
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }

    /// Attach dual-key grace end.
    pub fn with_grace_until(mut self, grace_until: Option<&str>) -> Self {
        self.grace_until = grace_until
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }

    /// Attach peer-delta issuer.
    pub fn with_issuer(mut self, issuer_id: Option<&str>) -> Self {
        self.issuer_id = issuer_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        self
    }
}

/// Append-only trust ceremony audit log helpers.
pub struct TrustAuditLog;

impl TrustAuditLog {
    /// Absolute path to the JSONL audit file under a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("identity").join(TRUST_AUDIT_FILE)
    }

    /// Append one entry (creates `identity/` as needed). Fail closed on I/O.
    pub fn append(root: impl AsRef<Path>, entry: &TrustAuditEntry) -> Result<(), CryptoError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| CryptoError::Io(e.to_string()))?;
        }
        let line = serde_json::to_string(entry).map_err(|e| CryptoError::Io(e.to_string()))?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| CryptoError::Io(e.to_string()))?;
        writeln!(f, "{line}").map_err(|e| CryptoError::Io(e.to_string()))?;
        f.flush().map_err(|e| CryptoError::Io(e.to_string()))?;
        Ok(())
    }

    /// Load all entries in file order (empty if missing).
    pub fn load(root: impl AsRef<Path>) -> Result<Vec<TrustAuditEntry>, CryptoError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let f = fs::File::open(&path).map_err(|e| CryptoError::Io(e.to_string()))?;
        let reader = BufReader::new(f);
        let mut out = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| CryptoError::Io(e.to_string()))?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let entry: TrustAuditEntry = serde_json::from_str(trimmed)
                .map_err(|e| CryptoError::Io(format!("trust-audit.jsonl line {}: {e}", idx + 1)))?;
            out.push(entry);
        }
        Ok(out)
    }
}

/// Convenience: build + append in one call.
pub fn record_trust_audit(
    root: impl AsRef<Path>,
    entry: TrustAuditEntry,
) -> Result<(), CryptoError> {
    TrustAuditLog::append(root, &entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn append_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let e1 = TrustAuditEntry::new(
            TrustAuditAction::Revoke,
            "aira:identity:peer-a",
            Some("cli"),
        )
        .unwrap()
        .with_reason(Some("compromised"));
        let e2 = TrustAuditEntry::new(
            TrustAuditAction::Rotate,
            "aira:identity:peer-a",
            Some("peer-delta"),
        )
        .unwrap()
        .with_new_id(Some("aira:identity:peer-a-v2"))
        .with_pubkey_hex(Some("ab".repeat(32).as_str()))
        .with_issuer(Some("aira:identity:issuer"));
        TrustAuditLog::append(root, &e1).unwrap();
        TrustAuditLog::append(root, &e2).unwrap();
        let loaded = TrustAuditLog::load(root).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].action, TrustAuditAction::Revoke);
        assert_eq!(loaded[0].reason.as_deref(), Some("compromised"));
        assert_eq!(loaded[1].action, TrustAuditAction::Rotate);
        assert_eq!(loaded[1].new_id.as_deref(), Some("aira:identity:peer-a-v2"));
        assert_eq!(loaded[1].public_key_hex.as_ref().unwrap().len(), 64);
        assert!(loaded[0].public_key_hex.is_none());
    }

    #[test]
    fn missing_file_is_empty() {
        let dir = tempdir().unwrap();
        assert!(TrustAuditLog::load(dir.path()).unwrap().is_empty());
    }
}
