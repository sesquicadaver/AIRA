//! Local capability-based Discovery registry (Issue #75).

use std::collections::HashMap;

use aira_object::{AiraRef, Signature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::envelope::{local_signature, ProtocolError, ScopeDescriptor};

/// Capability descriptor (capability-centric; never a Node).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    pub capability_id: AiraRef,
    pub capability_type: String,
    pub schema_version: String,
    pub provider_csu: AiraRef,
    pub input_artifact_types: Vec<String>,
    pub output_artifact_types: Vec<String>,
    #[serde(default)]
    pub constraints: Value,
    pub scope: ScopeDescriptor,
    pub policy_refs: Vec<AiraRef>,
    pub signature: Signature,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// Discovery query result — Capability + provider CSU (no Node).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscoveryHit {
    pub capability: CapabilityDescriptor,
    pub provider_csu: AiraRef,
}

/// Local in-process discovery registry (no global registry).
#[derive(Debug, Default)]
pub struct DiscoveryRegistry {
    /// capability_type → capabilities
    by_type: HashMap<String, Vec<CapabilityDescriptor>>,
}

impl DiscoveryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability offered by a CSU.
    pub fn register(&mut self, capability: CapabilityDescriptor) -> Result<(), ProtocolError> {
        if capability.provider_csu.as_str().contains(":node:") {
            return Err(ProtocolError::Schema(
                "discovery must register Capability with provider CSU, not Node".into(),
            ));
        }
        if capability.signature.signature_value.trim().is_empty() {
            return Err(ProtocolError::InvalidSignature);
        }
        self.by_type
            .entry(capability.capability_type.clone())
            .or_default()
            .push(capability);
        Ok(())
    }

    /// Query by capability type — returns Capability hits, never Nodes.
    pub fn query(&self, capability_type: &str) -> Vec<DiscoveryHit> {
        self.by_type
            .get(capability_type)
            .into_iter()
            .flatten()
            .map(|c| DiscoveryHit {
                provider_csu: c.provider_csu.clone(),
                capability: c.clone(),
            })
            .collect()
    }

    /// Helper to build a signed local capability.
    pub fn local_capability(
        capability_id: &str,
        capability_type: &str,
        provider_csu: &str,
    ) -> Result<CapabilityDescriptor, ProtocolError> {
        Ok(CapabilityDescriptor {
            capability_id: AiraRef::parse(capability_id)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            capability_type: capability_type.into(),
            schema_version: "0.1".into(),
            provider_csu: AiraRef::parse(provider_csu)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            input_artifact_types: vec!["ExecutionArtifact".into()],
            output_artifact_types: vec!["ExecutionArtifact".into()],
            constraints: serde_json::json!({}),
            scope: ScopeDescriptor::local("discovery"),
            policy_refs: vec![AiraRef::parse("aira:policy:default")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            signature: local_signature(),
            confidence: Some(1.0),
        })
    }
}
