//! Local capability-based Discovery registry (Issue #75 / Analyze-45 persist).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{AiraRef, Signature};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::envelope::{local_signature, ProtocolError, ScopeDescriptor};

/// On-disk schema tag for durable discovery registry.
pub const DISCOVERY_REGISTRY_SCHEMA: &str = "aira:discovery:registry:v1";

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

/// Durable file shape under `.aira/discovery/registry.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct DiscoveryFile {
    schema: String,
    capabilities: Vec<CapabilityDescriptor>,
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

    /// Path to durable registry JSON for a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("discovery").join("registry.json")
    }

    /// Load from disk or empty registry.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::new());
        }
        let raw = fs::read_to_string(&path).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let file: DiscoveryFile =
            serde_json::from_str(&raw).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        if file.schema != DISCOVERY_REGISTRY_SCHEMA {
            return Err(ProtocolError::Schema(format!(
                "discovery schema mismatch: {}",
                file.schema
            )));
        }
        let mut reg = Self::new();
        for cap in file.capabilities {
            reg.register(cap)?;
        }
        Ok(reg)
    }

    /// Persist all capabilities (creates `discovery/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), ProtocolError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        }
        let capabilities: Vec<CapabilityDescriptor> =
            self.list_all().into_iter().cloned().collect();
        let file = DiscoveryFile {
            schema: DISCOVERY_REGISTRY_SCHEMA.into(),
            capabilities,
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| ProtocolError::Schema(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| ProtocolError::Schema(e.to_string()))
    }

    /// True if a capability_id is already registered.
    pub fn contains(&self, capability_id: &str) -> bool {
        self.by_type
            .values()
            .flatten()
            .any(|c| c.capability_id.as_str() == capability_id)
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

    /// List all registered capabilities (sorted by capability_id).
    pub fn list_all(&self) -> Vec<&CapabilityDescriptor> {
        let mut caps: Vec<_> = self.by_type.values().flatten().collect();
        caps.sort_by(|a, b| a.capability_id.as_str().cmp(b.capability_id.as_str()));
        caps
    }

    /// Helper to build a signed local capability (also used by HTTP capability seed).
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let mut p = env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        p.push(format!("aira-discovery-{nanos}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn discovery_persist_roundtrip() {
        let root = temp_root();
        let mut reg = DiscoveryRegistry::new();
        let cap = DiscoveryRegistry::local_capability(
            "aira:capability:local:execution-basic",
            "local.execution-basic",
            "aira:csu:execution-basic",
        )
        .unwrap();
        reg.register(cap).unwrap();
        reg.save(&root).unwrap();
        assert!(DiscoveryRegistry::path(&root).exists());

        let loaded = DiscoveryRegistry::load(&root).unwrap();
        assert!(loaded.contains("aira:capability:local:execution-basic"));
        assert_eq!(loaded.list_all().len(), 1);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn discovery_load_empty_when_missing() {
        let root = temp_root();
        let loaded = DiscoveryRegistry::load(&root).unwrap();
        assert!(loaded.list_all().is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
