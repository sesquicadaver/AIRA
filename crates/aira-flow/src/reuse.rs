//! Durable reuse-index catalog (`problems/reuse-index.json`, RFC-0087 / `#204`).
//!
//! Keys are admission-scoped (`#326` / RFC-0211): [`AdmissionSnapshot::reuse_catalog_key`],
//! not bare problem-text SHA. Legacy text-only keys in on-disk indexes are ignored
//! (fail-closed → re-execute).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::admission::AdmissionSnapshot;

/// Persistent map: admission reuse key → reusable artifact id.
///
/// Field name retained for on-disk compatibility; values are no longer
/// text-only hashes after `#326`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ReuseIndex {
    pub by_content_hash: BTreeMap<String, String>,
}

pub(crate) fn load_reuse_index(path: &Path) -> Result<ReuseIndex, String> {
    if !path.exists() {
        return Ok(ReuseIndex::default());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

/// Look up a reusable artifact for this admit. `None` key (require-new) → miss.
pub(crate) fn lookup_artifact_id(
    path: &Path,
    admission: &AdmissionSnapshot,
) -> Result<Option<String>, String> {
    let Some(key) = admission.reuse_catalog_key() else {
        return Ok(None);
    };
    let idx = load_reuse_index(path)?;
    Ok(idx.by_content_hash.get(&key).cloned())
}

/// Record a Completed verified artifact under the admit's reuse key (`#326`).
///
/// No-op when `reuse_policy` is [`crate::ReusePolicy::RequireNewExecution`].
pub(crate) fn record_artifact_id(
    path: &Path,
    admission: &AdmissionSnapshot,
    artifact_id: &str,
) -> Result<(), String> {
    let Some(key) = admission.reuse_catalog_key() else {
        return Ok(());
    };
    let mut idx = load_reuse_index(path)?;
    idx.by_content_hash
        .entry(key)
        .or_insert_with(|| artifact_id.to_string());
    let raw = serde_json::to_string_pretty(&idx).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, raw).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::{AdmissionConstraints, ReusePolicy};
    use aira_object::ContentHash;

    /// Legacy helper: SHA of problem text alone (pre-`#326` key shape).
    fn problem_text_hash(text: &str) -> String {
        ContentHash::sha256_bytes(text.as_bytes())
            .as_str()
            .to_string()
    }

    #[test]
    fn lookup_misses_legacy_text_only_key() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let text = "Calculate 2 + 2";
        let legacy = problem_text_hash(text);
        let idx = serde_json::json!({
            "by_content_hash": { legacy: "aira:artifact:legacy" }
        });
        fs::write(&path, serde_json::to_string(&idx).unwrap()).unwrap();
        let snap = AdmissionSnapshot::default_for_text(text);
        assert!(lookup_artifact_id(&path, &snap).unwrap().is_none());
    }

    #[test]
    fn record_and_lookup_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let snap = AdmissionSnapshot::default_for_text("Calculate 2 + 2");
        record_artifact_id(&path, &snap, "aira:artifact:ready").unwrap();
        assert_eq!(
            lookup_artifact_id(&path, &snap).unwrap().as_deref(),
            Some("aira:artifact:ready")
        );
    }

    #[test]
    fn require_new_neither_records_nor_looks_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reuse-index.json");
        let snap = AdmissionSnapshot::from_text_and_constraints(
            "Calculate 2 + 2",
            &AdmissionConstraints {
                reuse_policy: ReusePolicy::RequireNewExecution,
                ..Default::default()
            },
        );
        record_artifact_id(&path, &snap, "aira:artifact:x").unwrap();
        assert!(!path.exists() || load_reuse_index(&path).unwrap().by_content_hash.is_empty());
        assert!(lookup_artifact_id(&path, &snap).unwrap().is_none());
    }
}
