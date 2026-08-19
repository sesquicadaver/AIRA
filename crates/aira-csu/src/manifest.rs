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
    /// Structural + cryptographic validation for registry admission.
    pub fn validate_for_registration(&self) -> Result<(), CsuError> {
        if self.signature.signature_value.trim().is_empty() {
            return Err(CsuError::UnsignedManifest(self.csu_id.clone()));
        }
        match self.verify_canonical() {
            Ok(()) => {}
            Err(aira_object::CryptoError::MissingOrLegacy) => {
                return Err(CsuError::UnsignedManifest(self.csu_id.clone()));
            }
            Err(other) => {
                return Err(CsuError::ManifestInvalid(format!("signature: {other}")));
            }
        }
        if self.abi_version != SUPPORTED_ABI_VERSION {
            return Err(CsuError::UnsupportedAbi(self.abi_version.clone()));
        }
        if self.csu_name.trim().is_empty() {
            return Err(CsuError::ManifestInvalid("csu_name empty".into()));
        }
        Ok(())
    }

    /// Sign over canonical JSON of this manifest without the top-level `signature`.
    ///
    /// Signer is `identity_ref` (process primary for basic manifests).
    pub fn attach_canonical_signature(mut self) -> Result<Self, aira_object::CryptoError> {
        self.resign_canonical()?;
        Ok(self)
    }

    /// Recompute the canonical signature in place using `identity_ref`.
    pub fn resign_canonical(&mut self) -> Result<(), aira_object::CryptoError> {
        let v = serde_json::to_value(&*self)
            .map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        self.signature = aira_object::sign_canonical_descriptor(&self.identity_ref, &v)?;
        Ok(())
    }

    /// Tenant-isolated sign over the same canonical message as [`Self::attach_canonical_signature`].
    pub fn attach_canonical_signature_for_tenant(
        mut self,
        tenant_csu: &AiraRef,
    ) -> Result<Self, aira_object::CryptoError> {
        let v =
            serde_json::to_value(&self).map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        let msg = aira_object::descriptor_signing_message(&v)?;
        self.signature = aira_object::signature_for_tenant(tenant_csu, &self.identity_ref, &msg)?;
        Ok(self)
    }

    /// Verify Ed25519 over canonical manifest hash. No LOCAL_TEST domain fallback.
    pub fn verify_canonical(&self) -> Result<(), aira_object::CryptoError> {
        let v =
            serde_json::to_value(self).map_err(|e| aira_object::CryptoError::Io(e.to_string()))?;
        aira_object::verify_canonical_descriptor(&self.signature, &v)
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
