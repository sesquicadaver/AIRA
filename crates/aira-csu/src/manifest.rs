//! CSU Manifest types (Schema Pack §15 / Issue #35).

use aira_object::{AiraRef, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::CsuError;

/// Supported Core ABI version for MVP.
pub const SUPPORTED_ABI_VERSION: &str = "0.1";

/// CSU type enum from schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CsuType {
    Context,
    Reduction,
    Evidence,
    Epistemic,
    Execution,
    Verification,
    Artifact,
    Discovery,
    Federation,
    Settlement,
    Optimization,
    PHM,
    Evolution,
    Research,
    HumanInteraction,
    Custom,
}

/// Sandbox declaration from manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CsuSandbox {
    pub filesystem: String,
    pub network: String,
    pub process: String,
    pub device_access: String,
    pub secret_access: String,
}

/// Capability descriptor (Schema Pack capability).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub capability_id: AiraRef,
    pub capability_type: String,
    pub schema_version: String,
    pub provider_csu: AiraRef,
    pub input_artifact_types: Vec<String>,
    pub output_artifact_types: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_context: Option<Vec<String>>,
    pub constraints: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_model_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_refs: Option<Vec<AiraRef>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    pub scope: Value,
    pub policy_refs: Vec<AiraRef>,
    pub signature: Signature,
}

/// Signed CSU manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CsuManifest {
    pub csu_id: AiraRef,
    pub csu_name: String,
    pub csu_type: CsuType,
    pub csu_version: String,
    pub abi_version: String,
    pub manifest_version: String,
    pub identity_ref: AiraRef,
    pub publisher_identity: AiraRef,
    pub capabilities: Vec<CapabilityDescriptor>,
    pub permissions: Vec<Value>,
    pub event_subscriptions: Vec<Value>,
    pub event_outputs: Vec<Value>,
    pub artifact_inputs: Vec<Value>,
    pub artifact_outputs: Vec<Value>,
    pub policy_refs: Vec<AiraRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_requirements: Option<Value>,
    pub sandbox: CsuSandbox,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifecycle_hooks: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_refs: Option<Vec<AiraRef>>,
    pub signature: Signature,
    pub created_at: Timestamp,
}

impl CsuManifest {
    /// Structural validation for registry admission (signature presence + ABI).
    pub fn validate_for_registration(&self) -> Result<(), CsuError> {
        if self.signature.signature_value.trim().is_empty() {
            return Err(CsuError::UnsignedManifest(self.csu_id.clone()));
        }
        if self.abi_version != SUPPORTED_ABI_VERSION {
            return Err(CsuError::UnsupportedAbi(self.abi_version.clone()));
        }
        if self.csu_name.trim().is_empty() {
            return Err(CsuError::ManifestInvalid("csu_name empty".into()));
        }
        Ok(())
    }

    /// Event type names this CSU subscribes to (from `event_subscriptions[].event_type`).
    pub fn subscribed_event_types(&self) -> Vec<String> {
        self.event_subscriptions
            .iter()
            .filter_map(|v| {
                v.get("event_type")
                    .and_then(|t| t.as_str())
                    .map(str::to_string)
            })
            .collect()
    }
}
