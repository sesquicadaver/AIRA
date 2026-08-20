//! Model compatibility resolver CSU (QUEUE #59 / Analyze-94).
//!
//! Classifies installed models as `runnable` / `incompatible` / `unknown` from a
//! local host profile + optional model profiles. Publishes
//! `CustomArtifact` payloads (`aira:schema:model:compatibility-evidence:0.1`).
//! No network, no download, not wired into C1.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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
pub const CSU_ID: &str = "aira:csu:model.compatibility";
/// Host hardware/backend profile (local descriptor, not Core).
pub const HOST_PROFILE_REL: &str = "models/host.profile.json";
/// Directory of optional model profile JSON files.
pub const MODEL_PROFILES_REL: &str = "models/profiles";
/// Inventory pointer written by `#58` scan.
pub const INVENTORY_POINTER_REL: &str = "models/inventory.latest.json";
/// Summary pointer for the latest compatibility run.
pub const COMPAT_POINTER_REL: &str = "models/compatibility.latest.json";

/// Resolver errors.
#[derive(Debug, Error)]
pub enum CompatError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("no inventory snapshot — run `aira models scan` first")]
    NoInventory,
    #[error("{0}")]
    Other(String),
}

/// Local host capability snapshot (not a Schema Pack Core type).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostProfile {
    pub available_vram_gb: f64,
    pub available_ram_gb: f64,
    pub available_disk_gb: f64,
    #[serde(default)]
    pub backends: Vec<String>,
}

/// Subset of model profile fields used for classification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelProfileView {
    pub model_ref: String,
    pub required_vram_gb: f64,
    pub required_ram_gb: f64,
    pub min_disk_gb: f64,
    #[serde(default)]
    pub supported_backends: Vec<String>,
}

/// Classification outcome for one model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compatibility {
    Runnable,
    Incompatible,
    Unknown,
}

impl Compatibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Runnable => "runnable",
            Self::Incompatible => "incompatible",
            Self::Unknown => "unknown",
        }
    }
}

/// One published evidence row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatRow {
    pub model_ref: String,
    pub compatibility: Compatibility,
    pub reason: String,
    pub confidence: f64,
    pub evidence_artifact_id: String,
}

/// Result of `resolve_and_publish`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatOutcome {
    pub rows: Vec<CompatRow>,
    pub summary_path: String,
}

/// Latest compatibility summary pointer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatPointer {
    pub updated_at: String,
    pub inventory_artifact_id: String,
    pub rows: Vec<CompatRow>,
}

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Signed manifest: read_only FS under models/, network none.
pub fn compatibility_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-compatibility".into(),
        csu_type: CsuType::Custom,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![json!({"filesystem": "read_only", "paths": ["models"]})],
        event_subscriptions: vec![json!({"event_type": "CustomEvent"})],
        event_outputs: vec![
            json!({"event_type": "CustomEvent"}),
            json!({"event_type": "ArtifactPublished"}),
        ],
        artifact_inputs: vec![json!({"artifact_type": "CustomArtifact"})],
        artifact_outputs: vec![json!({"artifact_type": "CustomArtifact"})],
        policy_refs: vec![AiraRef::parse("aira:policy:default").expect("policy")],
        resource_requirements: None,
        sandbox: CsuSandbox {
            filesystem: "read_only".into(),
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
    .expect("canonical compatibility manifest")
}

/// Classify one model against optional host + model profiles.
pub fn classify(
    host: Option<&HostProfile>,
    profile: Option<&ModelProfileView>,
) -> (Compatibility, String, f64) {
    match (host, profile) {
        (None, None) => (
            Compatibility::Unknown,
            "no host profile and no model profile".into(),
            0.2,
        ),
        (Some(_), None) => (
            Compatibility::Unknown,
            "no model profile for installed model".into(),
            0.4,
        ),
        (None, Some(_)) => (
            Compatibility::Unknown,
            "no host profile; cannot compare hardware".into(),
            0.4,
        ),
        (Some(h), Some(p)) => {
            if p.required_vram_gb > h.available_vram_gb {
                return (
                    Compatibility::Incompatible,
                    format!(
                        "required_vram_gb {} exceeds available {}",
                        p.required_vram_gb, h.available_vram_gb
                    ),
                    0.95,
                );
            }
            if p.required_ram_gb > h.available_ram_gb {
                return (
                    Compatibility::Incompatible,
                    format!(
                        "required_ram_gb {} exceeds available {}",
                        p.required_ram_gb, h.available_ram_gb
                    ),
                    0.95,
                );
            }
            if p.min_disk_gb > h.available_disk_gb {
                return (
                    Compatibility::Incompatible,
                    format!(
                        "min_disk_gb {} exceeds available {}",
                        p.min_disk_gb, h.available_disk_gb
                    ),
                    0.95,
                );
            }
            if !p.supported_backends.is_empty() {
                let overlap = p
                    .supported_backends
                    .iter()
                    .any(|b| h.backends.iter().any(|hb| hb == b));
                if !overlap {
                    return (
                        Compatibility::Incompatible,
                        format!(
                            "no overlapping backends (model={:?}, host={:?})",
                            p.supported_backends, h.backends
                        ),
                        0.9,
                    );
                }
            }
            (
                Compatibility::Runnable,
                "hardware and backends satisfy model profile".into(),
                0.9,
            )
        }
    }
}

fn load_host_profile(root: &Path) -> Result<Option<HostProfile>, CompatError> {
    let path = root.join(HOST_PROFILE_REL);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| CompatError::Io(e.to_string()))?;
    let hp: HostProfile =
        serde_json::from_str(&raw).map_err(|e| CompatError::Other(format!("host profile: {e}")))?;
    Ok(Some(hp))
}

fn load_model_profiles(root: &Path) -> Result<BTreeMap<String, ModelProfileView>, CompatError> {
    let dir = root.join(MODEL_PROFILES_REL);
    let mut out = BTreeMap::new();
    if !dir.exists() {
        return Ok(out);
    }
    for entry in WalkDir::new(&dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(path).map_err(|e| CompatError::Io(e.to_string()))?;
        let v: Value = serde_json::from_str(&raw).map_err(|e| CompatError::Other(e.to_string()))?;
        let Some(model_ref) = v.get("model_ref").and_then(|x| x.as_str()) else {
            continue;
        };
        let view = ModelProfileView {
            model_ref: model_ref.to_string(),
            required_vram_gb: v
                .get("required_vram_gb")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0),
            required_ram_gb: v
                .get("required_ram_gb")
                .and_then(|x| x.as_f64())
                .unwrap_or(0.0),
            min_disk_gb: v.get("min_disk_gb").and_then(|x| x.as_f64()).unwrap_or(0.0),
            supported_backends: v
                .get("supported_backends")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|i| i.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        };
        out.insert(view.model_ref.clone(), view);
    }
    Ok(out)
}

fn load_inventory_models(root: &Path) -> Result<(String, Vec<String>), CompatError> {
    let ppath = root.join(INVENTORY_POINTER_REL);
    if !ppath.exists() {
        return Err(CompatError::NoInventory);
    }
    let raw = fs::read_to_string(&ppath).map_err(|e| CompatError::Io(e.to_string()))?;
    let pointer: Value =
        serde_json::from_str(&raw).map_err(|e| CompatError::Other(e.to_string()))?;
    let artifact_id = pointer
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CompatError::Other("inventory pointer missing artifact_id".into()))?
        .to_string();
    let id = AiraRef::parse(&artifact_id).map_err(|e| CompatError::Other(e.to_string()))?;
    let store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| CompatError::Artifact(e.to_string()))?;
    let (_desc, bytes) = store
        .resolve(&id)
        .map_err(|e| CompatError::Artifact(e.to_string()))?;
    let payload: Value =
        serde_json::from_slice(&bytes).map_err(|e| CompatError::Other(e.to_string()))?;
    let models = payload
        .get("installed_models")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    Ok((artifact_id, models))
}

fn build_evidence_payload(
    host_ref: &AiraRef,
    model_ref: &str,
    profile_ref: Option<&str>,
    compatibility: Compatibility,
    reason: &str,
    confidence: f64,
    assessed_at: &str,
) -> Result<Value, CompatError> {
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:compatibility-evidence:0.1"),
    );
    body.insert("model_ref".into(), json!(model_ref));
    if let Some(pr) = profile_ref {
        body.insert("profile_ref".into(), json!(pr));
    }
    body.insert("host_ref".into(), json!(host_ref.as_str()));
    body.insert("compatibility".into(), json!(compatibility.as_str()));
    body.insert("reason".into(), json!(reason));
    body.insert("confidence".into(), json!(confidence));
    body.insert(
        "scope".into(),
        json!({"scope_type": "local", "description": "host-local compatibility assessment"}),
    );
    body.insert("assessed_at".into(), json!(assessed_at));
    body.insert("evidence_refs".into(), json!([]));

    let for_sign = Value::Object(body.clone());
    let bytes = serde_json::to_vec(&for_sign).map_err(|e| CompatError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| CompatError::Other(e.to_string()))?,
    );
    // Schema does not require signature on evidence payload — strip before validate if needed.
    // Keep signature out of schema-required object: remove to match schema additionalProperties.
    body.remove("signature");
    Ok(Value::Object(body))
}

fn append_custom_event(root: &Path, event: EventDescriptor) -> Result<(), CompatError> {
    let path = root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| CompatError::Io(e.to_string()))?;
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
        fs::create_dir_all(parent).map_err(|e| CompatError::Io(e.to_string()))?;
    }
    let json = serde_json::to_string_pretty(&log).map_err(|e| CompatError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| CompatError::Io(e.to_string()))?;
    Ok(())
}

/// Resolve installed models and publish one evidence artifact per model.
pub fn resolve_and_publish(aira_root: impl AsRef<Path>) -> Result<CompatOutcome, CompatError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let (inventory_id, models) = load_inventory_models(root)?;
    let host = load_host_profile(root)?;
    let profiles = load_model_profiles(root)?;
    let assessed_at = utc_now_rfc3339().map_err(|e| CompatError::Crypto(e.to_string()))?;
    let host_ref = active_identity();

    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| CompatError::Artifact(e.to_string()))?;
    let mut rows = Vec::new();

    for model_ref in models {
        let profile = profiles.get(&model_ref);
        let (compat, reason, confidence) = classify(host.as_ref(), profile);
        let profile_ref = profile.map(|p| format!("aira:profile:local:{}", p.model_ref));
        let payload = build_evidence_payload(
            &host_ref,
            &model_ref,
            profile_ref.as_deref(),
            compat,
            &reason,
            confidence,
            &assessed_at,
        )?;

        if let Ok(schema_root) = aira_schema::find_repo_root(
            std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()),
        ) {
            if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
                reg.validate("aira:schema:model:compatibility-evidence:0.1", &payload)
                    .map_err(|e| CompatError::Schema(e.to_string()))?;
            }
        }

        let bytes = json_bytes(&payload);
        let content_hash = ContentHash::sha256_bytes(&bytes);
        let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
        let artifact_id = format!("aira:artifact:compat:{hash_hex}");
        let desc = make_artifact(
            &artifact_id,
            ArtifactType::CustomArtifact,
            &bytes,
            vec![AiraRef::parse(CSU_ID).expect("csu")],
        );
        match store.publish(desc, &bytes) {
            Ok(_) => {}
            Err(aira_artifact::ArtifactError::Immutable(_)) => {}
            Err(e) => return Err(CompatError::Artifact(e.to_string())),
        }

        let ev_id = format!("aira:event:compat-{}", &hash_hex[..16.min(hash_hex.len())]);
        let event = make_event(
            &ev_id,
            EventType::CustomEvent,
            vec![],
            vec![AiraRef::parse(&artifact_id).expect("aid")],
            vec![],
            Some(format!("op:compatibility-assessed:{artifact_id}")),
        );
        append_custom_event(root, event)?;

        rows.push(CompatRow {
            model_ref,
            compatibility: compat,
            reason,
            confidence,
            evidence_artifact_id: artifact_id,
        });
    }

    let pointer = CompatPointer {
        updated_at: assessed_at,
        inventory_artifact_id: inventory_id,
        rows: rows.clone(),
    };
    let spath = root.join(COMPAT_POINTER_REL);
    if let Some(parent) = spath.parent() {
        fs::create_dir_all(parent).map_err(|e| CompatError::Io(e.to_string()))?;
    }
    fs::write(
        &spath,
        serde_json::to_string_pretty(&pointer).map_err(|e| CompatError::Other(e.to_string()))?,
    )
    .map_err(|e| CompatError::Io(e.to_string()))?;

    Ok(CompatOutcome {
        rows,
        summary_path: spath.display().to_string(),
    })
}

/// Load the latest compatibility summary.
pub fn load_latest_summary(aira_root: impl AsRef<Path>) -> Result<CompatPointer, CompatError> {
    let path = aira_root.as_ref().join(COMPAT_POINTER_REL);
    if !path.exists() {
        return Err(CompatError::Other(
            "no compatibility summary — run `aira models compatible` first".into(),
        ));
    }
    let raw = fs::read_to_string(&path).map_err(|e| CompatError::Io(e.to_string()))?;
    serde_json::from_str(&raw).map_err(|e| CompatError::Other(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
    use aira_csu::support::{json_bytes, make_artifact};
    use aira_object::ContentHash;

    fn init_min_root(root: &Path) {
        for d in [
            "artifacts",
            "events",
            "models",
            "models/profiles",
            "identity",
        ] {
            fs::create_dir_all(root.join(d)).unwrap();
        }
        fs::write(
            root.join("events").join("event-log.json"),
            "{\"events\":[]}",
        )
        .unwrap();
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    fn seed_inventory(root: &Path, model_refs: &[&str]) {
        let payload = json!({
            "payload_schema": "aira:schema:model:inventory:0.1",
            "host_ref": "aira:identity:local-test",
            "installed_models": model_refs,
            "runnable_models": [],
            "downloadable_compatible_models": [],
            "incompatible_models": [],
            "cache_budget": {"total_gb": 1.0, "used_gb": 0.0, "reserved_gb": 0.0},
            "updated_at": "2026-08-20T05:00:00Z",
            "signature": {
                "algorithm": "ed25519",
                "key_ref": "aira:identity:local-test",
                "signature_value": "TESTSIG"
            }
        });
        // Inventory publish requires real signature for CAS — use make_artifact path with signed desc.
        // For pointer we only need CAS payload bytes resolvable; use make_artifact with local-test.
        let bytes = json_bytes(&payload);
        let hash = ContentHash::sha256_bytes(&bytes);
        let hex = hash.as_str().trim_start_matches("sha256:");
        let aid = format!("aira:artifact:inv:{hex}");
        let desc = make_artifact(&aid, ArtifactType::CustomArtifact, &bytes, vec![]);
        let mut store = CasArtifactStore::open(root.join("artifacts")).unwrap();
        store.publish(desc, &bytes).unwrap();
        let pointer = json!({
            "artifact_id": aid,
            "content_hash": hash.as_str(),
            "updated_at": "2026-08-20T05:00:00Z"
        });
        fs::write(
            root.join(INVENTORY_POINTER_REL),
            serde_json::to_string_pretty(&pointer).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_read_only_no_network() {
        let m = compatibility_manifest();
        assert_eq!(m.sandbox.filesystem, "read_only");
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.csu_type, CsuType::Custom);
    }

    #[test]
    fn classify_runnable_and_incompatible() {
        let host = HostProfile {
            available_vram_gb: 8.0,
            available_ram_gb: 32.0,
            available_disk_gb: 100.0,
            backends: vec!["llama.cpp".into()],
        };
        let ok = ModelProfileView {
            model_ref: "aira:model:a".into(),
            required_vram_gb: 6.0,
            required_ram_gb: 12.0,
            min_disk_gb: 4.0,
            supported_backends: vec!["llama.cpp".into()],
        };
        let (c, reason, _) = classify(Some(&host), Some(&ok));
        assert_eq!(c, Compatibility::Runnable);
        assert!(reason.contains("satisfy"));

        let bad = ModelProfileView {
            required_vram_gb: 24.0,
            ..ok.clone()
        };
        let (c2, reason2, _) = classify(Some(&host), Some(&bad));
        assert_eq!(c2, Compatibility::Incompatible);
        assert!(reason2.contains("required_vram_gb"));
    }

    #[test]
    fn resolve_publishes_evidence_without_download() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let model =
            "aira:model:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        seed_inventory(dir.path(), &[model]);
        fs::write(
            dir.path().join(HOST_PROFILE_REL),
            r#"{"available_vram_gb":8,"available_ram_gb":32,"available_disk_gb":100,"backends":["llama.cpp"]}"#,
        )
        .unwrap();
        fs::write(
            dir.path().join(MODEL_PROFILES_REL).join("m.json"),
            format!(
                r#"{{"model_ref":"{model}","required_vram_gb":6,"required_ram_gb":12,"min_disk_gb":4,"supported_backends":["llama.cpp"],"supported_quantizations":["int4"],"context_length":2048,"modalities":["text"],"domains":["general"],"estimated_latency_class":"interactive","evidence_refs":[],"payload_schema":"aira:schema:model:profile:0.1"}}"#
            ),
        )
        .unwrap();

        let out = resolve_and_publish(dir.path()).unwrap();
        assert_eq!(out.rows.len(), 1);
        assert_eq!(out.rows[0].compatibility, Compatibility::Runnable);
        assert!(!out.rows[0].evidence_artifact_id.is_empty());
        assert!(!out.rows[0].reason.is_empty());

        let summary = load_latest_summary(dir.path()).unwrap();
        assert_eq!(summary.rows.len(), 1);
        assert_eq!(summary.rows[0].compatibility, Compatibility::Runnable);
    }

    #[test]
    fn unknown_without_profiles() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let model =
            "aira:model:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        seed_inventory(dir.path(), &[model]);
        let out = resolve_and_publish(dir.path()).unwrap();
        assert_eq!(out.rows[0].compatibility, Compatibility::Unknown);
    }
}
