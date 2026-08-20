//! Local Model Inventory CSU (QUEUE #58 / Analyze-93).
//!
//! Read-only scan of a **scoped** directory under the node root. Publishes an
//! immutable `CustomArtifact` payload (`aira:schema:model:inventory:0.1`).
//! Does not download, open network sockets, or join the C1 plane.

use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, local_identity, make_artifact, make_event, mvp_timestamp};
use aira_csu::{CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};
use aira_event::{EventDescriptor, EventType};
use aira_object::{
    active_identity, active_signature, utc_now_rfc3339, AiraRef, ContentHash, Signature,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use thiserror::Error;
use walkdir::WalkDir;

/// Stable CSU id for registry / RFC-D.
pub const CSU_ID: &str = "aira:csu:model.inventory";
/// Default scoped relative directory under the AIRA node root.
pub const DEFAULT_SCOPED_REL: &str = "models";
/// Pointer to the latest inventory snapshot.
pub const LATEST_POINTER_REL: &str = "models/inventory.latest.json";
/// Weight extensions discovered by scan (no download).
const MODEL_EXTS: &[&str] = &["gguf", "safetensors"];

/// Inventory CSU errors.
#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("path outside scoped filesystem: {0}")]
    OutsideScope(String),
    #[error("scoped path is not a directory: {0}")]
    NotDirectory(String),
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("no inventory snapshot — run `aira models scan` first")]
    NoSnapshot,
    #[error("{0}")]
    Other(String),
}

/// One local weight file discovered under the scoped root.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScannedModel {
    pub model_ref: String,
    pub path: String,
    pub bytes: u64,
    pub content_hash: String,
}

/// Result of a successful scan + publish.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOutcome {
    pub artifact_id: String,
    pub content_hash: String,
    pub installed_count: usize,
    pub payload: Value,
}

/// Pointer written after each successful scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryPointer {
    pub artifact_id: String,
    pub content_hash: String,
    pub updated_at: String,
}

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Absolute scoped root: `<aira_root>/models`.
pub fn scoped_root(aira_root: impl AsRef<Path>) -> PathBuf {
    aira_root.as_ref().join(DEFAULT_SCOPED_REL)
}

/// Ensure `candidate` resolves inside the scoped root (canonical path check).
pub fn ensure_within_scope(
    aira_root: impl AsRef<Path>,
    candidate: impl AsRef<Path>,
) -> Result<PathBuf, InventoryError> {
    let scope = scoped_root(aira_root.as_ref());
    fs::create_dir_all(&scope).map_err(|e| InventoryError::Io(e.to_string()))?;
    let scope_canon = scope
        .canonicalize()
        .map_err(|e| InventoryError::Io(format!("{}: {e}", scope.display())))?;
    let cand = candidate.as_ref();
    if !cand.exists() {
        fs::create_dir_all(cand).map_err(|e| InventoryError::Io(e.to_string()))?;
    }
    let cand_canon = cand
        .canonicalize()
        .map_err(|e| InventoryError::Io(format!("{}: {e}", cand.display())))?;
    if !cand_canon.starts_with(&scope_canon) {
        return Err(InventoryError::OutsideScope(cand.display().to_string()));
    }
    if !cand_canon.is_dir() {
        return Err(InventoryError::NotDirectory(cand.display().to_string()));
    }
    Ok(cand_canon)
}

/// Signed manifest: `filesystem=scoped`, `network=none` (not a basic CSU).
pub fn inventory_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-inventory".into(),
        csu_type: CsuType::Custom,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![json!({"filesystem": "scoped", "paths": [DEFAULT_SCOPED_REL]})],
        event_subscriptions: vec![json!({"event_type": "CustomEvent"})],
        event_outputs: vec![
            json!({"event_type": "CustomEvent"}),
            json!({"event_type": "ArtifactPublished"}),
        ],
        artifact_inputs: vec![],
        artifact_outputs: vec![json!({"artifact_type": "CustomArtifact"})],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        resource_requirements: None,
        sandbox: CsuSandbox {
            filesystem: "scoped".into(),
            network: "none".into(),
            process: "in_process".into(),
            device_access: "none".into(),
            secret_access: "none".into(),
        },
        lifecycle_hooks: None,
        provenance_refs: None,
        signature: aira_csu::support::local_signature(),
        created_at: mvp_timestamp(),
    }
    .attach_canonical_signature()
    .expect("canonical inventory manifest")
}

/// Discover local weight files under `scan_dir` (must already be within scope).
pub fn scan_weight_files(scan_dir: &Path) -> Result<Vec<ScannedModel>, InventoryError> {
    let mut found = Vec::new();
    for entry in WalkDir::new(scan_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if !MODEL_EXTS.iter().any(|x| *x == ext) {
            continue;
        }
        let bytes = fs::read(path).map_err(|e| InventoryError::Io(e.to_string()))?;
        let hash = ContentHash::sha256_bytes(&bytes);
        let hex = hash.as_str().trim_start_matches("sha256:");
        let model_ref = format!("aira:model:sha256:{hex}");
        found.push(ScannedModel {
            model_ref,
            path: path.display().to_string(),
            bytes: bytes.len() as u64,
            content_hash: hash.as_str().to_string(),
        });
    }
    found.sort_by(|a, b| a.model_ref.cmp(&b.model_ref));
    Ok(found)
}

/// Build inventory payload (unsigned fields + attached payload signature).
pub fn build_inventory_payload(
    host_ref: &AiraRef,
    models: &[ScannedModel],
    updated_at: &str,
) -> Result<Value, InventoryError> {
    let used_gb = models.iter().map(|m| m.bytes as f64).sum::<f64>() / 1_000_000_000.0;
    let installed: Vec<Value> = models
        .iter()
        .map(|m| Value::String(m.model_ref.clone()))
        .collect();

    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:inventory:0.1"),
    );
    body.insert("host_ref".into(), json!(host_ref.as_str()));
    body.insert("installed_models".into(), Value::Array(installed));
    // Compatibility classification is QUEUE #59 — leave empty here.
    body.insert("runnable_models".into(), json!([]));
    body.insert("downloadable_compatible_models".into(), json!([]));
    body.insert("incompatible_models".into(), json!([]));
    body.insert(
        "cache_budget".into(),
        json!({
            "total_gb": used_gb.max(1.0),
            "used_gb": used_gb,
            "reserved_gb": 0.0
        }),
    );
    body.insert("updated_at".into(), json!(updated_at));

    let for_sign = Value::Object(body.clone());
    let bytes = serde_json::to_vec(&for_sign).map_err(|e| InventoryError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| InventoryError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn pointer_path(aira_root: &Path) -> PathBuf {
    aira_root.join(LATEST_POINTER_REL)
}

fn append_custom_event(aira_root: &Path, event: EventDescriptor) -> Result<(), InventoryError> {
    let path = aira_root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| InventoryError::Io(e.to_string()))?;
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        EventLogFile::default()
    };
    if !log
        .events
        .iter()
        .any(|e| e.event_id.as_str() == event.event_id.as_str())
    {
        log.events.push(event);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| InventoryError::Io(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(&log).map_err(|e| InventoryError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| InventoryError::Io(e.to_string()))?;
    Ok(())
}

/// Scan scoped FS, publish immutable inventory artifact, update latest pointer.
///
/// `scan_rel_or_abs` defaults to `<root>/models`. Path must stay within scoped root.
pub fn scan_and_publish(
    aira_root: impl AsRef<Path>,
    scan_dir: Option<&Path>,
) -> Result<ScanOutcome, InventoryError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let default_scope = scoped_root(root);
    let target = match scan_dir {
        Some(p) => ensure_within_scope(root, p)?,
        None => ensure_within_scope(root, &default_scope)?,
    };

    let models = scan_weight_files(&target)?;
    let updated_at = utc_now_rfc3339().map_err(|e| InventoryError::Crypto(e.to_string()))?;
    let host = active_identity();
    let payload = build_inventory_payload(&host, &models, &updated_at)?;

    // Optional schema gate when schemas/ is reachable from cwd/repo.
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:inventory:0.1", &payload)
                .map_err(|e| InventoryError::Schema(e.to_string()))?;
        }
    }

    let bytes = json_bytes(&payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:inv:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );

    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| InventoryError::Artifact(e.to_string()))?;
    match store.publish(desc.clone(), &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {
            // Same content already published — still refresh pointer.
        }
        Err(e) => return Err(InventoryError::Artifact(e.to_string())),
    }

    let pointer = InventoryPointer {
        artifact_id: artifact_id.clone(),
        content_hash: content_hash.as_str().to_string(),
        updated_at: updated_at.clone(),
    };
    let ppath = pointer_path(root);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| InventoryError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer).map_err(|e| InventoryError::Other(e.to_string()))?,
    )
    .map_err(|e| InventoryError::Io(e.to_string()))?;

    let ev_id = format!("aira:event:inv-{}", &hash_hex[..16.min(hash_hex.len())]);
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:inventory-updated:{artifact_id}")),
    );
    append_custom_event(root, event)?;

    Ok(ScanOutcome {
        artifact_id,
        content_hash: content_hash.as_str().to_string(),
        installed_count: models.len(),
        payload,
    })
}

/// Load the latest inventory payload published by [`scan_and_publish`].
pub fn load_latest(
    aira_root: impl AsRef<Path>,
) -> Result<(InventoryPointer, Value), InventoryError> {
    let root = aira_root.as_ref();
    let ppath = pointer_path(root);
    if !ppath.exists() {
        return Err(InventoryError::NoSnapshot);
    }
    let raw = fs::read_to_string(&ppath).map_err(|e| InventoryError::Io(e.to_string()))?;
    let pointer: InventoryPointer =
        serde_json::from_str(&raw).map_err(|e| InventoryError::Other(e.to_string()))?;
    let id =
        AiraRef::parse(&pointer.artifact_id).map_err(|e| InventoryError::Other(e.to_string()))?;
    let store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| InventoryError::Artifact(e.to_string()))?;
    let (_desc, bytes) = store
        .resolve(&id)
        .map_err(|e| InventoryError::Artifact(e.to_string()))?;
    let payload: Value =
        serde_json::from_slice(&bytes).map_err(|e| InventoryError::Other(e.to_string()))?;
    Ok((pointer, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_min_root(root: &Path) {
        for d in ["artifacts", "events", "models", "identity"] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(
            root.join("events").join("event-log.json"),
            "{\"events\":[]}",
        )
        .unwrap();
        // Minimal config marker so CLI ensure_init would pass if reused.
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_declares_scoped_fs_no_network() {
        let m = inventory_manifest();
        assert_eq!(m.sandbox.filesystem, "scoped");
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.csu_type, CsuType::Custom);
        assert_ne!(m.sandbox.filesystem, "none");
    }

    #[test]
    fn rejects_path_outside_scope() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let outside = tempfile::tempdir().unwrap();
        let err = ensure_within_scope(dir.path(), outside.path()).unwrap_err();
        assert!(matches!(err, InventoryError::OutsideScope(_)));
    }

    #[test]
    fn scan_publish_list_roundtrip_no_download() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let models = scoped_root(dir.path());
        fs::create_dir_all(&models).unwrap();
        fs::write(models.join("tiny.gguf"), b"GGUF-fake-weights").unwrap();
        fs::write(models.join("readme.txt"), b"ignore").unwrap();

        let out = scan_and_publish(dir.path(), None).unwrap();
        assert_eq!(out.installed_count, 1);
        assert_eq!(
            out.payload.get("payload_schema").and_then(|v| v.as_str()),
            Some("aira:schema:model:inventory:0.1")
        );
        assert_eq!(
            out.payload
                .get("downloadable_compatible_models")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(0)
        );

        let (ptr, payload) = load_latest(dir.path()).unwrap();
        assert_eq!(ptr.artifact_id, out.artifact_id);
        let installed = payload
            .get("installed_models")
            .and_then(|v| v.as_array())
            .unwrap();
        assert_eq!(installed.len(), 1);
        assert!(installed[0]
            .as_str()
            .unwrap()
            .starts_with("aira:model:sha256:"));
    }
}
