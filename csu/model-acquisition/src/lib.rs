//! Model acquisition: policy gate, quarantine, verify, activate, share gate (QUEUE #60–#66).
//!
//! Default-deny download/share; ALLOW paths authorize later steps without performing them.
//! Inventory refresh is CLI-orchestrated (no CSU↛CSU). Not wired into C1. `network=none`.

use std::fs;
use std::path::{Path, PathBuf};

use aira_artifact::{ArtifactStore, ArtifactType, CasArtifactStore};
use aira_csu::support::{json_bytes, local_identity, make_artifact, make_event, mvp_timestamp};
use aira_csu::{CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};
use aira_event::{EventDescriptor, EventType};
use aira_object::{
    active_identity, active_signature, is_cryptographic_signature, utc_now_rfc3339, verify_ed25519,
    AiraRef, ContentHash, Signature,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use thiserror::Error;

/// Stable CSU id.
pub const CSU_ID: &str = "aira:csu:model.acquisition";
/// Optional on-disk acquisition policy payload (schema `acquisition-policy:0.1`).
pub const POLICY_FILE_REL: &str = "models/acquisition.policy.json";
/// Latest DENY/ALLOW decision pointer (download).
pub const DECISION_POINTER_REL: &str = "models/acquisition.decision.latest.json";
/// Latest DENY/ALLOW decision pointer (publish/share).
pub const SHARE_DECISION_POINTER_REL: &str = "models/share.decision.latest.json";
/// Quarantine directory under scoped models tree.
pub const QUARANTINE_REL: &str = "models/quarantine";
/// Latest quarantine fetch pointer.
pub const QUARANTINE_POINTER_REL: &str = "models/quarantine.latest.json";
/// Verified staging directory (post-hash/signature check; pre-activate).
pub const VERIFIED_REL: &str = "models/verified";
/// Latest verified staging pointer.
pub const VERIFIED_POINTER_REL: &str = "models/verified.latest.json";
/// Activated model cache (post-activate; inventory scans this tree).
pub const CACHE_REL: &str = "models/cache";
/// Latest activation pointer.
pub const ACTIVATED_POINTER_REL: &str = "models/activated.latest.json";

/// Acquisition gate / quarantine errors.
#[derive(Debug, Error)]
pub enum AcquisitionError {
    #[error("io: {0}")]
    Io(String),
    #[error("artifact: {0}")]
    Artifact(String),
    #[error("crypto: {0}")]
    Crypto(String),
    #[error("schema: {0}")]
    Schema(String),
    #[error("remote source rejected (local --source only): {0}")]
    RemoteSource(String),
    #[error("source is not a file: {0}")]
    SourceNotFile(String),
    #[error("source not found: {0}")]
    SourceMissing(String),
    #[error("quarantine path outside scoped models: {0}")]
    OutsideScope(String),
    #[error("no quarantine snapshot — run download --source first")]
    NoQuarantine,
    #[error("no verified snapshot — run models verify first")]
    NoVerified,
    #[error("model artifact missing or invalid: {0}")]
    BadArtifact(String),
    #[error("{0}")]
    Other(String),
}

/// Gate decision — ALLOW authorizes a future transfer; gate alone never copies bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GateDecision {
    Allow,
    Deny,
}

impl GateDecision {
    /// Stable uppercase label for CLI / pointer JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "ALLOW",
            Self::Deny => "DENY",
        }
    }
}

/// Outcome of a download request evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcquireOutcome {
    pub decision: GateDecision,
    pub reason: String,
    pub reason_ref: String,
    pub model_ref: String,
    pub decision_artifact_id: String,
    pub policy_present: bool,
    pub auto_download: Option<bool>,
}

/// Outcome of a publish/share request evaluation (no ShareOffer bytes).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOutcome {
    pub decision: GateDecision,
    pub reason: String,
    pub reason_ref: String,
    pub model_ref: String,
    pub decision_artifact_id: String,
    pub policy_present: bool,
    pub share_custom_models: Option<bool>,
}

/// Result of gate + optional local quarantine copy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchOutcome {
    /// Policy DENY — no bytes copied.
    Denied(AcquireOutcome),
    /// Policy ALLOW and local source copied into quarantine.
    Quarantined {
        gate: AcquireOutcome,
        quarantine_path: String,
        bytes: u64,
        content_hash: String,
        source_path: String,
    },
}

/// Pointer to the latest policy decision artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPointer {
    pub updated_at: String,
    pub decision: String,
    pub model_ref: String,
    pub reason: String,
    pub decision_artifact_id: String,
}

/// Pointer to the latest quarantine object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinePointer {
    pub updated_at: String,
    pub model_ref: String,
    pub quarantine_path: String,
    pub source_path: String,
    pub bytes: u64,
    pub content_hash: String,
    pub decision_artifact_id: String,
}

/// Pointer to the latest verified staging object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedPointer {
    pub updated_at: String,
    pub model_ref: String,
    pub verified_path: String,
    pub quarantine_path: String,
    pub content_hash: String,
    pub evidence_artifact_id: String,
}

/// Pointer to the latest activated cache object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivatedPointer {
    pub updated_at: String,
    pub model_ref: String,
    pub cache_path: String,
    pub verified_path: String,
    pub content_hash: String,
    pub evidence_artifact_id: String,
}

/// Result of explicit activation (no model execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateOutcome {
    pub model_ref: String,
    pub cache_path: String,
    pub verified_path: String,
    pub content_hash: String,
    pub evidence_artifact_id: String,
    /// Absolute path to `models/cache` for inventory scan orchestration.
    pub cache_scan_dir: String,
}

/// Result of quarantine hash/signature verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerifyOutcome {
    /// Hash mismatch or unsigned/invalid signature — weights remain in quarantine.
    Rejected {
        model_ref: String,
        quarantine_path: String,
        observed_hash: String,
        expected_hash: Option<String>,
        reason: String,
        reason_ref: String,
        evidence_artifact_id: String,
    },
    /// Hash + signature OK — copied to `models/verified/` (not activated).
    Verified {
        model_ref: String,
        quarantine_path: String,
        verified_path: String,
        content_hash: String,
        evidence_artifact_id: String,
    },
}

/// Loaded acquisition policy view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PolicyView {
    pub auto_download: bool,
    pub allow_untrusted_models: bool,
    pub share_custom_models: bool,
}

/// Crate version for smoke tests.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Signed manifest: scoped models FS; no network.
pub fn acquisition_manifest() -> CsuManifest {
    CsuManifest {
        csu_id: AiraRef::parse(CSU_ID).expect("csu_id"),
        csu_name: "model-acquisition".into(),
        csu_type: CsuType::Custom,
        csu_version: "0.1.0".into(),
        abi_version: SUPPORTED_ABI_VERSION.into(),
        manifest_version: "0.1".into(),
        identity_ref: local_identity(),
        publisher_identity: local_identity(),
        capabilities: vec![],
        permissions: vec![json!({"filesystem": "scoped", "paths": ["models"]})],
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
    .expect("canonical acquisition manifest")
}

/// Load optional local acquisition policy file.
pub fn load_policy(aira_root: impl AsRef<Path>) -> Result<Option<PolicyView>, AcquisitionError> {
    let path = aira_root.as_ref().join(POLICY_FILE_REL);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let v: Value =
        serde_json::from_str(&raw).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    if let Ok(schema_root) = aira_schema::find_repo_root(
        std::env::current_dir().unwrap_or_else(|_| aira_root.as_ref().to_path_buf()),
    ) {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:acquisition-policy:0.1", &v)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    Ok(Some(PolicyView {
        auto_download: v
            .get("auto_download")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        allow_untrusted_models: v
            .get("allow_untrusted_models")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        share_custom_models: v
            .get("share_custom_models")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    }))
}

/// Evaluate a download request; publish decision evidence. Gate alone never copies weights.
///
/// Rules:
/// - no policy → DENY (`aira:reason:no-acquisition-policy`)
/// - `auto_download=false` → DENY (`aira:reason:auto-download-false`)
/// - `auto_download=true` → ALLOW (`aira:reason:auto-download-true`)
pub fn request_download(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<AcquireOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let policy = load_policy(root)?;

    let (decision, reason, reason_ref, auto_download) = match &policy {
        None => (
            GateDecision::Deny,
            "no acquisition policy; default DENY".to_string(),
            "aira:reason:no-acquisition-policy".to_string(),
            None,
        ),
        Some(p) if !p.auto_download => (
            GateDecision::Deny,
            "auto_download=false; download DENY".to_string(),
            "aira:reason:auto-download-false".to_string(),
            Some(false),
        ),
        Some(p) => (
            GateDecision::Allow,
            "auto_download=true; download ALLOW (no transfer in this step)".to_string(),
            "aira:reason:auto-download-true".to_string(),
            Some(p.auto_download),
        ),
    };

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let decision_payload = build_decision(decision, &reason_ref)?;
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:policy:decision:0.1", &decision_payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }

    let bytes = json_bytes(&decision_payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let kind = match decision {
        GateDecision::Allow => "acq-allow",
        GateDecision::Deny => "acq-deny",
    };
    let artifact_id = format!("aira:artifact:{kind}:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }

    let op = match decision {
        GateDecision::Allow => "policy-allowed",
        GateDecision::Deny => "policy-denied",
    };
    let ev_id = format!("aira:event:{kind}-{}", &hash_hex[..16.min(hash_hex.len())]);
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:{op}:download:{model_ref}:{reason_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = DecisionPointer {
        updated_at,
        decision: decision.as_str().into(),
        model_ref: model_ref.to_string(),
        reason: reason.clone(),
        decision_artifact_id: artifact_id.clone(),
    };
    let ppath = root.join(DECISION_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    let _host = active_identity();

    Ok(AcquireOutcome {
        decision,
        reason,
        reason_ref,
        model_ref: model_ref.to_string(),
        decision_artifact_id: artifact_id,
        policy_present: policy.is_some(),
        auto_download,
    })
}

/// Evaluate a publish/share request; publish decision evidence. Gate alone never writes ShareOffer.
///
/// Rules:
/// - no policy → DENY (`aira:reason:no-acquisition-policy`)
/// - `share_custom_models=false` → DENY (`aira:reason:share-custom-models-false`)
/// - `share_custom_models=true` → ALLOW (`aira:reason:share-custom-models-true`)
pub fn request_publish(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
) -> Result<ShareOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let policy = load_policy(root)?;

    let (decision, reason, reason_ref, share_custom_models) = match &policy {
        None => (
            GateDecision::Deny,
            "no acquisition policy; publish DENY".to_string(),
            "aira:reason:no-acquisition-policy".to_string(),
            None,
        ),
        Some(p) if !p.share_custom_models => (
            GateDecision::Deny,
            "share_custom_models=false; publish DENY".to_string(),
            "aira:reason:share-custom-models-false".to_string(),
            Some(false),
        ),
        Some(p) => (
            GateDecision::Allow,
            "share_custom_models=true; publish ALLOW (no ShareOffer in this step)".to_string(),
            "aira:reason:share-custom-models-true".to_string(),
            Some(p.share_custom_models),
        ),
    };

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let decision_payload = build_decision(decision, &reason_ref)?;
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:policy:decision:0.1", &decision_payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }

    let bytes = json_bytes(&decision_payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let kind = match decision {
        GateDecision::Allow => "share-allow",
        GateDecision::Deny => "share-deny",
    };
    let artifact_id = format!("aira:artifact:{kind}:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }

    let op = match decision {
        GateDecision::Allow => "policy-allowed",
        GateDecision::Deny => "policy-denied",
    };
    let ev_id = format!("aira:event:{kind}-{}", &hash_hex[..16.min(hash_hex.len())]);
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:{op}:publish:{model_ref}:{reason_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = DecisionPointer {
        updated_at,
        decision: decision.as_str().into(),
        model_ref: model_ref.to_string(),
        reason: reason.clone(),
        decision_artifact_id: artifact_id.clone(),
    };
    let ppath = root.join(SHARE_DECISION_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(ShareOutcome {
        decision,
        reason,
        reason_ref,
        model_ref: model_ref.to_string(),
        decision_artifact_id: artifact_id,
        policy_present: policy.is_some(),
        share_custom_models,
    })
}

/// Run policy gate; on ALLOW copy local `source` into scoped quarantine.
///
/// Does **not** verify hash/signature (`#63`) or activate (`#64`). Rejects URL schemes.
pub fn fetch_to_quarantine(
    aira_root: impl AsRef<Path>,
    model_ref: &str,
    source: impl AsRef<Path>,
) -> Result<FetchOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let source = source.as_ref();
    reject_remote_source(source)?;

    let gate = request_download(root, model_ref)?;
    if gate.decision == GateDecision::Deny {
        return Ok(FetchOutcome::Denied(gate));
    }

    if !source.exists() {
        return Err(AcquisitionError::SourceMissing(
            source.display().to_string(),
        ));
    }
    if !source.is_file() {
        return Err(AcquisitionError::SourceNotFile(
            source.display().to_string(),
        ));
    }

    let file_name = source
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("source has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(model_ref);
    let dest_dir = root.join(QUARANTINE_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;

    let dest = dest_dir.join(&file_name);
    fs::copy(source, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let file_bytes = fs::read(&dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let bytes = file_bytes.len() as u64;
    let content_hash = ContentHash::sha256_bytes(&file_bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;

    let receipt = build_quarantine_receipt(
        model_ref,
        &dest.display().to_string(),
        &source.display().to_string(),
        bytes,
        content_hash.as_str(),
        &gate.decision_artifact_id,
    )?;
    let receipt_bytes = json_bytes(&receipt);
    let receipt_hash = ContentHash::sha256_bytes(&receipt_bytes);
    let receipt_hex = receipt_hash.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-quarantine:{receipt_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &receipt_bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &receipt_bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }

    let ev_id = format!(
        "aira:event:acq-quarantine-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&artifact_id).expect("aid")],
        vec![],
        Some(format!("op:quarantine-fetched:download:{model_ref}")),
    );
    append_custom_event(root, event)?;

    let pointer = QuarantinePointer {
        updated_at,
        model_ref: model_ref.to_string(),
        quarantine_path: dest.display().to_string(),
        source_path: source.display().to_string(),
        bytes,
        content_hash: content_hash.as_str().to_string(),
        decision_artifact_id: gate.decision_artifact_id.clone(),
    };
    let ppath = root.join(QUARANTINE_POINTER_REL);
    if let Some(parent) = ppath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &ppath,
        serde_json::to_string_pretty(&pointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(FetchOutcome::Quarantined {
        gate,
        quarantine_path: dest.display().to_string(),
        bytes,
        content_hash: content_hash.as_str().to_string(),
        source_path: source.display().to_string(),
    })
}

/// Verify quarantined weights against a ModelArtifact payload (hash + signature).
///
/// On reject: Evidence + Event; quarantine file left in place.
/// On pass: copy into `models/verified/` (no activate / inventory promote).
pub fn verify_quarantine(
    aira_root: impl AsRef<Path>,
    artifact_path: impl AsRef<Path>,
) -> Result<VerifyOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);

    let qpath = root.join(QUARANTINE_POINTER_REL);
    if !qpath.exists() {
        return Err(AcquisitionError::NoQuarantine);
    }
    let pointer: QuarantinePointer = serde_json::from_str(
        &fs::read_to_string(&qpath).map_err(|e| AcquisitionError::Io(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Other(e.to_string()))?;

    let qfile = Path::new(&pointer.quarantine_path);
    if !qfile.is_file() {
        return Err(AcquisitionError::SourceMissing(
            pointer.quarantine_path.clone(),
        ));
    }
    ensure_under_models(root, qfile)?;

    let file_bytes = fs::read(qfile).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let observed = ContentHash::sha256_bytes(&file_bytes);
    let observed_hash = observed.as_str().to_string();

    let art_raw = fs::read_to_string(artifact_path.as_ref())
        .map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
    let artifact: Value =
        serde_json::from_str(&art_raw).map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:artifact:0.1", &artifact)
                .map_err(|e| AcquisitionError::BadArtifact(e.to_string()))?;
        }
    }

    let expected_hash = artifact
        .get("content_hash")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let sig_val = artifact.get("signature").cloned();
    let signature: Option<Signature> = match sig_val {
        Some(v) => Some(
            serde_json::from_value(v)
                .map_err(|e| AcquisitionError::BadArtifact(format!("signature: {e}")))?,
        ),
        None => None,
    };

    let mut reject_reason: Option<(String, String)> = None;

    match &signature {
        None => {
            reject_reason = Some((
                "model artifact has no signature".into(),
                "aira:reason:model-unsigned".into(),
            ));
        }
        Some(sig) if !is_cryptographic_signature(sig) => {
            reject_reason = Some((
                "model artifact signature is not cryptographic (unsigned/legacy)".into(),
                "aira:reason:model-unsigned".into(),
            ));
        }
        Some(sig) => {
            let msg = signing_bytes_without_signature(&artifact)?;
            if verify_ed25519(sig, &msg).is_err() {
                reject_reason = Some((
                    "model artifact signature verification failed".into(),
                    "aira:reason:model-signature-invalid".into(),
                ));
            }
        }
    }

    if reject_reason.is_none() {
        match &expected_hash {
            Some(exp) if exp != &observed_hash => {
                reject_reason = Some((
                    format!("content_hash mismatch: expected {exp}, observed {observed_hash}"),
                    "aira:reason:model-hash-mismatch".into(),
                ));
            }
            None => {
                reject_reason = Some((
                    "model artifact missing content_hash".into(),
                    "aira:reason:model-hash-mismatch".into(),
                ));
            }
            Some(_) => {}
        }
    }

    if let Some((reason, reason_ref)) = reject_reason {
        let evidence_id = publish_verify_evidence(
            root,
            VerifyEvidenceInput {
                model_ref: &pointer.model_ref,
                verified: false,
                quarantine_path: &pointer.quarantine_path,
                verified_path: None,
                observed_hash: &observed_hash,
                expected_hash: expected_hash.as_deref(),
                reason_ref: &reason_ref,
            },
        )?;
        let ev_id = format!(
            "aira:event:acq-verify-reject-{}",
            &observed_hash.trim_start_matches("sha256:")
                [..16.min(observed_hash.trim_start_matches("sha256:").len())]
        );
        let event = make_event(
            &ev_id,
            EventType::CustomEvent,
            vec![],
            vec![AiraRef::parse(&evidence_id).expect("aid")],
            vec![],
            Some(format!(
                "op:verify-rejected:download:{}:{reason_ref}",
                pointer.model_ref
            )),
        );
        append_custom_event(root, event)?;
        return Ok(VerifyOutcome::Rejected {
            model_ref: pointer.model_ref,
            quarantine_path: pointer.quarantine_path,
            observed_hash,
            expected_hash,
            reason,
            reason_ref,
            evidence_artifact_id: evidence_id,
        });
    }

    let file_name = qfile
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("quarantine path has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(&pointer.model_ref);
    let dest_dir = root.join(VERIFIED_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;
    let dest = dest_dir.join(&file_name);
    fs::copy(qfile, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let dest_display = dest.display().to_string();
    let reason_ref = "aira:reason:model-verified";
    let evidence_id = publish_verify_evidence(
        root,
        VerifyEvidenceInput {
            model_ref: &pointer.model_ref,
            verified: true,
            quarantine_path: &pointer.quarantine_path,
            verified_path: Some(&dest_display),
            observed_hash: &observed_hash,
            expected_hash: expected_hash.as_deref(),
            reason_ref,
        },
    )?;
    let hash_hex = observed_hash.trim_start_matches("sha256:");
    let ev_id = format!(
        "aira:event:acq-verify-ok-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&evidence_id).expect("aid")],
        vec![],
        Some(format!(
            "op:verify-passed:download:{}:{reason_ref}",
            pointer.model_ref
        )),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let vpointer = VerifiedPointer {
        updated_at,
        model_ref: pointer.model_ref.clone(),
        verified_path: dest.display().to_string(),
        quarantine_path: pointer.quarantine_path.clone(),
        content_hash: observed_hash.clone(),
        evidence_artifact_id: evidence_id.clone(),
    };
    let vpath = root.join(VERIFIED_POINTER_REL);
    if let Some(parent) = vpath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &vpath,
        serde_json::to_string_pretty(&vpointer)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    Ok(VerifyOutcome::Verified {
        model_ref: pointer.model_ref,
        quarantine_path: pointer.quarantine_path,
        verified_path: dest.display().to_string(),
        content_hash: observed_hash,
        evidence_artifact_id: evidence_id,
    })
}

/// Explicitly activate a verified model into local cache.
///
/// Copies `models/verified/…` → `models/cache/…`, publishes ModelInstalled-style
/// Evidence + Event. Does **not** execute the model. Inventory refresh is left to
/// the CLI (`scan_and_publish` on [`CACHE_REL`]) to respect CSU↛CSU firewall.
pub fn activate_verified(aira_root: impl AsRef<Path>) -> Result<ActivateOutcome, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);

    let vpath = root.join(VERIFIED_POINTER_REL);
    if !vpath.exists() {
        return Err(AcquisitionError::NoVerified);
    }
    let pointer: VerifiedPointer = serde_json::from_str(
        &fs::read_to_string(&vpath).map_err(|e| AcquisitionError::Io(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Other(e.to_string()))?;

    let vfile = Path::new(&pointer.verified_path);
    if !vfile.is_file() {
        return Err(AcquisitionError::SourceMissing(
            pointer.verified_path.clone(),
        ));
    }
    ensure_under_models(root, vfile)?;

    let file_name = vfile
        .file_name()
        .ok_or_else(|| AcquisitionError::Other("verified path has no file name".into()))?
        .to_string_lossy()
        .to_string();
    let slot = sanitize_slot(&pointer.model_ref);
    let dest_dir = root.join(CACHE_REL).join(&slot);
    fs::create_dir_all(&dest_dir).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest_dir)?;
    let dest = dest_dir.join(&file_name);
    fs::copy(vfile, &dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    ensure_under_models(root, &dest)?;

    let file_bytes = fs::read(&dest).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let content_hash = ContentHash::sha256_bytes(&file_bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let dest_display = dest.display().to_string();

    let evidence_id = publish_activate_evidence(
        root,
        &pointer.model_ref,
        &pointer.verified_path,
        &dest_display,
        content_hash.as_str(),
    )?;

    let ev_id = format!(
        "aira:event:acq-activate-{}",
        &hash_hex[..16.min(hash_hex.len())]
    );
    let event = make_event(
        &ev_id,
        EventType::CustomEvent,
        vec![],
        vec![AiraRef::parse(&evidence_id).expect("aid")],
        vec![],
        Some(format!("op:model-installed:activate:{}", pointer.model_ref)),
    );
    append_custom_event(root, event)?;

    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let ap = ActivatedPointer {
        updated_at,
        model_ref: pointer.model_ref.clone(),
        cache_path: dest_display.clone(),
        verified_path: pointer.verified_path.clone(),
        content_hash: content_hash.as_str().to_string(),
        evidence_artifact_id: evidence_id.clone(),
    };
    let apath = root.join(ACTIVATED_POINTER_REL);
    if let Some(parent) = apath.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &apath,
        serde_json::to_string_pretty(&ap).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;

    let cache_scan = root.join(CACHE_REL);
    Ok(ActivateOutcome {
        model_ref: pointer.model_ref,
        cache_path: dest_display,
        verified_path: pointer.verified_path,
        content_hash: content_hash.as_str().to_string(),
        evidence_artifact_id: evidence_id,
        cache_scan_dir: cache_scan.display().to_string(),
    })
}

fn publish_activate_evidence(
    root: &Path,
    model_ref: &str,
    verified_path: &str,
    cache_path: &str,
    content_hash: &str,
) -> Result<String, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-installed-evidence"));
    body.insert("model_ref".into(), json!(model_ref));
    body.insert("verified".into(), json!(true));
    body.insert("activated".into(), json!(true));
    body.insert("executed".into(), json!(false));
    body.insert("verified_path".into(), json!(verified_path));
    body.insert("cache_path".into(), json!(cache_path));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert("reason_refs".into(), json!(["aira:reason:model-activated"]));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    let payload = Value::Object(body);
    let bytes = json_bytes(&payload);
    let ch = ContentHash::sha256_bytes(&bytes);
    let hash_hex = ch.as_str().trim_start_matches("sha256:");
    let artifact_id = format!("aira:artifact:acq-activate:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }
    Ok(artifact_id)
}

fn signing_bytes_without_signature(artifact: &Value) -> Result<Vec<u8>, AcquisitionError> {
    let obj = artifact
        .as_object()
        .ok_or_else(|| AcquisitionError::BadArtifact("artifact must be object".into()))?;
    let mut body = Map::new();
    for (k, v) in obj {
        if k != "signature" {
            body.insert(k.clone(), v.clone());
        }
    }
    serde_json::to_vec(&Value::Object(body)).map_err(|e| AcquisitionError::Other(e.to_string()))
}

struct VerifyEvidenceInput<'a> {
    model_ref: &'a str,
    verified: bool,
    quarantine_path: &'a str,
    verified_path: Option<&'a str>,
    observed_hash: &'a str,
    expected_hash: Option<&'a str>,
    reason_ref: &'a str,
}

fn publish_verify_evidence(
    root: &Path,
    input: VerifyEvidenceInput<'_>,
) -> Result<String, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-verify-evidence"));
    body.insert("model_ref".into(), json!(input.model_ref));
    body.insert("verified".into(), json!(input.verified));
    body.insert("activated".into(), json!(false));
    body.insert("quarantine_path".into(), json!(input.quarantine_path));
    if let Some(p) = input.verified_path {
        body.insert("verified_path".into(), json!(p));
    }
    body.insert("observed_hash".into(), json!(input.observed_hash));
    if let Some(e) = input.expected_hash {
        body.insert("expected_hash".into(), json!(e));
    }
    body.insert("reason_refs".into(), json!([input.reason_ref]));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    let payload = Value::Object(body);
    let bytes = json_bytes(&payload);
    let content_hash = ContentHash::sha256_bytes(&bytes);
    let hash_hex = content_hash.as_str().trim_start_matches("sha256:");
    let kind = if input.verified {
        "acq-verify-ok"
    } else {
        "acq-verify-reject"
    };
    let artifact_id = format!("aira:artifact:{kind}:{hash_hex}");
    let desc = make_artifact(
        &artifact_id,
        ArtifactType::CustomArtifact,
        &bytes,
        vec![AiraRef::parse(CSU_ID).expect("csu")],
    );
    let mut store = CasArtifactStore::open(root.join("artifacts"))
        .map_err(|e| AcquisitionError::Artifact(e.to_string()))?;
    match store.publish(desc, &bytes) {
        Ok(_) => {}
        Err(aira_artifact::ArtifactError::Immutable(_)) => {}
        Err(e) => return Err(AcquisitionError::Artifact(e.to_string())),
    }
    Ok(artifact_id)
}

fn reject_remote_source(source: &Path) -> Result<(), AcquisitionError> {
    let s = source.to_string_lossy();
    let lower = s.to_ascii_lowercase();
    for scheme in ["http://", "https://", "ftp://", "sftp://"] {
        if lower.starts_with(scheme) {
            return Err(AcquisitionError::RemoteSource(s.to_string()));
        }
    }
    Ok(())
}

fn sanitize_slot(model_ref: &str) -> String {
    let mut out = String::with_capacity(model_ref.len());
    for c in model_ref.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "model".into()
    } else {
        out
    }
}

fn ensure_under_models(root: &Path, path: &Path) -> Result<(), AcquisitionError> {
    let models = root.join("models");
    fs::create_dir_all(&models).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let scope = models
        .canonicalize()
        .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    let canon = path
        .canonicalize()
        .map_err(|e| AcquisitionError::Io(format!("{}: {e}", path.display())))?;
    if !canon.starts_with(&scope) {
        return Err(AcquisitionError::OutsideScope(path.display().to_string()));
    }
    Ok(())
}

fn build_quarantine_receipt(
    model_ref: &str,
    quarantine_path: &str,
    source_path: &str,
    bytes: u64,
    content_hash: &str,
    decision_artifact_id: &str,
) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("kind".into(), json!("model-quarantine-receipt"));
    body.insert("model_ref".into(), json!(model_ref));
    body.insert("quarantine_path".into(), json!(quarantine_path));
    body.insert("source_path".into(), json!(source_path));
    body.insert("bytes".into(), json!(bytes));
    body.insert("content_hash".into(), json!(content_hash));
    body.insert("decision_artifact_id".into(), json!(decision_artifact_id));
    body.insert("verified".into(), json!(false));
    body.insert("activated".into(), json!(false));
    let for_sign = Value::Object(body.clone());
    let raw = serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&raw);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn build_decision(decision: GateDecision, reason_ref: &str) -> Result<Value, AcquisitionError> {
    let mut body = Map::new();
    body.insert("decision".into(), json!(decision.as_str()));
    body.insert("requirements".into(), json!([]));
    body.insert("reason_refs".into(), json!([reason_ref]));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    Ok(Value::Object(body))
}

fn append_custom_event(root: &Path, event: EventDescriptor) -> Result<(), AcquisitionError> {
    let path = root.join("events").join("event-log.json");
    #[derive(Serialize, Deserialize, Default)]
    struct EventLogFile {
        events: Vec<EventDescriptor>,
    }
    let mut log: EventLogFile = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| AcquisitionError::Io(e.to_string()))?;
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
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    let json =
        serde_json::to_string_pretty(&log).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    fs::write(&path, json).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(())
}

/// Write an acquisition policy file (for `policy set`).
///
/// Convenience: `share_custom_models=false`.
pub fn write_default_deny_policy(
    aira_root: impl AsRef<Path>,
    auto_download: bool,
) -> Result<PathBuf, AcquisitionError> {
    write_acquisition_policy(aira_root, auto_download, false)
}

/// Write acquisition policy with explicit download and share flags.
pub fn write_acquisition_policy(
    aira_root: impl AsRef<Path>,
    auto_download: bool,
    share_custom_models: bool,
) -> Result<PathBuf, AcquisitionError> {
    let root = aira_root.as_ref();
    let _ = aira_object::register_node_identity(root);
    let updated_at = utc_now_rfc3339().map_err(|e| AcquisitionError::Crypto(e.to_string()))?;
    let host = active_identity();
    let mut body = Map::new();
    body.insert(
        "payload_schema".into(),
        json!("aira:schema:model:acquisition-policy:0.1"),
    );
    body.insert("host_ref".into(), json!(host.as_str()));
    body.insert("auto_download".into(), json!(auto_download));
    body.insert("allow_untrusted_models".into(), json!(false));
    body.insert("share_custom_models".into(), json!(share_custom_models));
    body.insert("updated_at".into(), json!(updated_at));
    let for_sign = Value::Object(body.clone());
    let bytes =
        serde_json::to_vec(&for_sign).map_err(|e| AcquisitionError::Other(e.to_string()))?;
    let sig: Signature = active_signature(&bytes);
    body.insert(
        "signature".into(),
        serde_json::to_value(&sig).map_err(|e| AcquisitionError::Other(e.to_string()))?,
    );
    let payload = Value::Object(body);
    if let Ok(schema_root) =
        aira_schema::find_repo_root(std::env::current_dir().unwrap_or_else(|_| root.to_path_buf()))
    {
        if let Ok(reg) = aira_schema::SchemaRegistry::load(schema_root.join("schemas")) {
            reg.validate("aira:schema:model:acquisition-policy:0.1", &payload)
                .map_err(|e| AcquisitionError::Schema(e.to_string()))?;
        }
    }
    let path = root.join(POLICY_FILE_REL);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| AcquisitionError::Io(e.to_string()))?;
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&payload)
            .map_err(|e| AcquisitionError::Other(e.to_string()))?,
    )
    .map_err(|e| AcquisitionError::Io(e.to_string()))?;
    Ok(path)
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
        fs::write(
            root.join("config.json"),
            r#"{"node":{"mode":"local","profile":"C1"},"security":{"allow_network_for_csu":false,"allow_shell_for_csu":false,"require_signed_artifacts":true,"require_signed_events":true,"require_signed_csu_manifests":true},"storage":{"object_store":"sqlite","event_log":"json","artifact_store":"filesystem"},"csu":{"autoload":[]}}"#,
        )
        .unwrap();
    }

    fn weight_files(root: &Path) -> Vec<String> {
        fs::read_dir(root.join("models"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().to_string();
                if n.ends_with(".gguf") || n.ends_with(".safetensors") {
                    Some(n)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_network_none_scoped_fs() {
        let m = acquisition_manifest();
        assert_eq!(m.sandbox.network, "none");
        assert_eq!(m.sandbox.filesystem, "scoped");
        assert_eq!(m.csu_type, CsuType::Custom);
    }

    #[test]
    fn deny_without_policy_emits_decision() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = request_download(
            dir.path(),
            "aira:model:sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(!out.policy_present);
        assert!(out.reason.contains("no acquisition policy"));
        assert!(dir.path().join(DECISION_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        assert!(!events.is_empty());
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-denied:download:"));
    }

    #[test]
    fn deny_when_auto_download_false() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), false).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert_eq!(out.auto_download, Some(false));
        assert!(out.reason.contains("auto_download=false"));
    }

    #[test]
    fn allow_when_auto_download_true_no_transfer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let out = request_download(dir.path(), "aira:model:example").unwrap();
        assert_eq!(out.decision, GateDecision::Allow);
        assert_eq!(out.auto_download, Some(true));
        assert_eq!(out.reason_ref, "aira:reason:auto-download-true");
        assert!(out.decision_artifact_id.contains("acq-allow"));
        let pointer: DecisionPointer = serde_json::from_str(
            &fs::read_to_string(dir.path().join(DECISION_POINTER_REL)).unwrap(),
        )
        .unwrap();
        assert_eq!(pointer.decision, "ALLOW");
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let payload = events
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-allowed:download:"));
        assert!(weight_files(dir.path()).is_empty());
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_fetch_after_allow_copies_local_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("outside-weights.gguf");
        fs::write(&src, b"fake-gguf-bytes").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:example", &src).unwrap();
        match out {
            FetchOutcome::Quarantined {
                gate,
                quarantine_path,
                bytes,
                ..
            } => {
                assert_eq!(gate.decision, GateDecision::Allow);
                assert_eq!(bytes, 15);
                assert!(Path::new(&quarantine_path).exists());
                assert!(quarantine_path.contains("quarantine"));
                assert!(!quarantine_path.contains("verified"));
            }
            FetchOutcome::Denied(_) => panic!("expected quarantine"),
        }
        assert!(dir.path().join(QUARANTINE_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let events = log.get("events").and_then(|e| e.as_array()).unwrap();
        let joined: String = events
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:quarantine-fetched:download:"));
        // Not activated into inventory cache root as loose weight.
        assert!(weight_files(dir.path()).is_empty());
    }

    #[test]
    fn quarantine_denied_without_policy_no_copy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let src = dir.path().join("x.gguf");
        fs::write(&src, b"data").unwrap();
        let out = fetch_to_quarantine(dir.path(), "aira:model:x", &src).unwrap();
        assert!(matches!(out, FetchOutcome::Denied(_)));
        assert!(!dir.path().join("models/quarantine").exists());
    }

    #[test]
    fn quarantine_rejects_http_source() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let err = fetch_to_quarantine(
            dir.path(),
            "aira:model:x",
            Path::new("https://example.com/m.gguf"),
        )
        .unwrap_err();
        assert!(matches!(err, AcquisitionError::RemoteSource(_)));
    }

    fn signed_model_artifact(model_id: &str, content_hash: &str) -> Value {
        let mut body = Map::new();
        body.insert(
            "payload_schema".into(),
            json!("aira:schema:model:artifact:0.1"),
        );
        body.insert("model_id".into(), json!(model_id));
        body.insert("format".into(), json!("gguf"));
        body.insert("quantization".into(), json!("int4"));
        body.insert("parameter_class".into(), json!("7B"));
        body.insert("content_hash".into(), json!(content_hash));
        body.insert(
            "provenance_refs".into(),
            json!(["aira:identity:local-test"]),
        );
        let for_sign = Value::Object(body.clone());
        let raw = serde_json::to_vec(&for_sign).unwrap();
        let sig = active_signature(&raw);
        body.insert("signature".into(), serde_json::to_value(&sig).unwrap());
        Value::Object(body)
    }

    #[test]
    fn verify_promotes_to_verified_on_match() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("ok.gguf");
        fs::write(&src, b"verify-me-please").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:ok", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"verify-me-please");
        let art = signed_model_artifact("aira:model:ok", observed.as_str());
        let art_path = dir.path().join("model.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Verified {
                verified_path,
                content_hash,
                ..
            } => {
                assert!(Path::new(&verified_path).exists());
                assert!(verified_path.contains("verified"));
                assert_eq!(content_hash, observed.as_str());
                // Quarantine retained.
                assert!(dir.path().join(QUARANTINE_POINTER_REL).exists());
            }
            VerifyOutcome::Rejected { reason, .. } => panic!("unexpected reject: {reason}"),
        }
        assert!(dir.path().join(VERIFIED_POINTER_REL).exists());
    }

    #[test]
    fn verify_rejects_hash_mismatch_keeps_quarantine() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("bad.gguf");
        fs::write(&src, b"actual-bytes").unwrap();
        let fetch = fetch_to_quarantine(dir.path(), "aira:model:bad", &src).unwrap();
        let qpath = match fetch {
            FetchOutcome::Quarantined {
                quarantine_path, ..
            } => quarantine_path,
            FetchOutcome::Denied(_) => panic!("expected quarantine"),
        };
        let wrong = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let art = signed_model_artifact("aira:model:bad", wrong);
        let art_path = dir.path().join("bad.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Rejected {
                reason_ref,
                quarantine_path,
                ..
            } => {
                assert_eq!(reason_ref, "aira:reason:model-hash-mismatch");
                assert_eq!(quarantine_path, qpath);
                assert!(Path::new(&qpath).exists());
            }
            VerifyOutcome::Verified { .. } => panic!("expected reject"),
        }
        assert!(!dir.path().join(VERIFIED_POINTER_REL).exists());
    }

    #[test]
    fn verify_rejects_unsigned_testsig() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("u.gguf");
        fs::write(&src, b"unsigned-bytes").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:u", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"unsigned-bytes");
        let mut art = signed_model_artifact("aira:model:u", observed.as_str());
        art.as_object_mut().unwrap().insert(
            "signature".into(),
            json!({
                "algorithm": "ed25519",
                "key_ref": "aira:identity:local-test",
                "signature_value": "TESTSIG"
            }),
        );
        let art_path = dir.path().join("u.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        let out = verify_quarantine(dir.path(), &art_path).unwrap();
        match out {
            VerifyOutcome::Rejected { reason_ref, .. } => {
                assert_eq!(reason_ref, "aira:reason:model-unsigned");
            }
            VerifyOutcome::Verified { .. } => panic!("expected unsigned reject"),
        }
    }

    #[test]
    fn activate_copies_verified_to_cache_no_execution() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_default_deny_policy(dir.path(), true).unwrap();
        let src = dir.path().join("act.gguf");
        fs::write(&src, b"activate-bytes").unwrap();
        fetch_to_quarantine(dir.path(), "aira:model:act", &src).unwrap();
        let observed = ContentHash::sha256_bytes(b"activate-bytes");
        let art = signed_model_artifact("aira:model:act", observed.as_str());
        let art_path = dir.path().join("act.artifact.json");
        fs::write(&art_path, serde_json::to_string_pretty(&art).unwrap()).unwrap();
        verify_quarantine(dir.path(), &art_path).unwrap();
        let out = activate_verified(dir.path()).unwrap();
        assert!(Path::new(&out.cache_path).exists());
        assert!(out.cache_path.contains("cache"));
        assert!(dir.path().join(ACTIVATED_POINTER_REL).exists());
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let joined: String = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .iter()
            .filter_map(|e| e.get("payload_ref").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join("|");
        assert!(joined.contains("op:model-installed:activate:"));
    }

    #[test]
    fn activate_requires_verified_pointer() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let err = activate_verified(dir.path()).unwrap_err();
        assert!(matches!(err, AcquisitionError::NoVerified));
    }

    #[test]
    fn publish_deny_without_policy() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert!(!out.policy_present);
        assert!(dir.path().join(SHARE_DECISION_POINTER_REL).exists());
    }

    #[test]
    fn publish_deny_when_share_false() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), false, false).unwrap();
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Deny);
        assert_eq!(out.share_custom_models, Some(false));
        assert_eq!(out.reason_ref, "aira:reason:share-custom-models-false");
    }

    #[test]
    fn publish_allow_when_share_true_no_offer_bytes() {
        let dir = tempfile::tempdir().unwrap();
        init_min_root(dir.path());
        write_acquisition_policy(dir.path(), false, true).unwrap();
        let out = request_publish(dir.path(), "aira:model:share-me").unwrap();
        assert_eq!(out.decision, GateDecision::Allow);
        assert_eq!(out.share_custom_models, Some(true));
        assert!(out.decision_artifact_id.contains("share-allow"));
        let log: Value = serde_json::from_str(
            &fs::read_to_string(dir.path().join("events/event-log.json")).unwrap(),
        )
        .unwrap();
        let payload = log
            .get("events")
            .and_then(|e| e.as_array())
            .unwrap()
            .last()
            .unwrap()
            .get("payload_ref")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(payload.contains("op:policy-allowed:publish:"));
        // No ShareOffer JSON written under models/.
        assert!(!dir.path().join("models/share-offer.latest.json").exists());
    }
}
