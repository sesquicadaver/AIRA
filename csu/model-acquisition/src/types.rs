use serde::{Deserialize, Serialize};

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
/// Latest local ShareOffer pointer.
pub const SHARE_OFFER_POINTER_REL: &str = "models/share-offer.latest.json";
/// Latest local capability advertisement pointer.
pub const CAPABILITY_AD_POINTER_REL: &str = "models/capability-ad.latest.json";

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

/// Result of gate + optional local ShareOffer materialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PublishOutcome {
    /// Policy DENY — no descriptors written.
    Denied(ShareOutcome),
    /// Policy ALLOW and local ModelArtifact + ShareOffer + capability ad published to CAS.
    Published {
        gate: ShareOutcome,
        model_artifact_id: String,
        share_offer_artifact_id: String,
        offer_id: String,
        capability_artifact_id: String,
        capability_id: String,
        content_hash: String,
        visibility: String,
        cache_path: String,
    },
}

/// Pointer to the latest local ShareOffer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareOfferPointer {
    pub updated_at: String,
    pub model_ref: String,
    pub offer_id: String,
    pub model_artifact_id: String,
    pub share_offer_artifact_id: String,
    pub content_hash: String,
    pub visibility: String,
    pub cache_path: String,
}

/// Pointer to the latest local capability advertisement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityAdPointer {
    pub updated_at: String,
    pub model_ref: String,
    pub capability_id: String,
    pub capability_artifact_id: String,
    pub share_offer_artifact_id: String,
    pub model_artifact_id: String,
    pub scope_type: String,
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
