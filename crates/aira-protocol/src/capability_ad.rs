//! Local Capability Advertisement persist (Book II §9 / B2-005; QUEUE #160).
//!
//! Durable store under `.aira/capability/advertisements.json`. Distinct from
//! DiscoveryRegistry (`CapabilityDescriptor` query surface): this is the CAP
//! advertisement record with `advertisement_id` + validity window.

use std::fs;
use std::path::{Path, PathBuf};

use aira_object::{AiraRef, Signature, Timestamp};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::envelope::{local_signature, mvp_timestamp, ProtocolError, ScopeDescriptor};

/// On-disk schema tag for durable capability advertisements.
pub const CAPABILITY_AD_STORE_SCHEMA: &str = "aira:capability:advertisements:v1";

/// Nested capability body inside an advertisement (Book II §9.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityAdBody {
    pub capability_type: String,
    pub schema_version: String,
    #[serde(default)]
    pub constraints: Value,
    pub scope: ScopeDescriptor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_model_ref: Option<AiraRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_refs: Vec<AiraRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// Validity window for an advertisement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityWindow {
    pub from: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<Timestamp>,
}

/// Book II Capability Advertisement (local reference; no network protocol).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityAdvertisement {
    pub advertisement_id: String,
    pub provider_csu: AiraRef,
    pub capability: CapabilityAdBody,
    pub policy_refs: Vec<AiraRef>,
    pub validity_window: ValidityWindow,
    pub signature: Signature,
}

/// Durable file shape under `.aira/capability/advertisements.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct CapabilityAdFile {
    schema: String,
    advertisements: Vec<CapabilityAdvertisement>,
}

/// Local in-process store of capability advertisements.
#[derive(Debug, Default)]
pub struct CapabilityAdvertisementStore {
    ads: Vec<CapabilityAdvertisement>,
}

impl CapabilityAdvertisementStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Path to durable advertisements JSON for a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("capability").join("advertisements.json")
    }

    /// Load from disk or empty store.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let path = Self::path(&root);
        if !path.exists() {
            return Ok(Self::new());
        }
        let raw = fs::read_to_string(&path).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let file: CapabilityAdFile =
            serde_json::from_str(&raw).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        if file.schema != CAPABILITY_AD_STORE_SCHEMA {
            return Err(ProtocolError::Schema(format!(
                "capability ad schema mismatch: {}",
                file.schema
            )));
        }
        let mut store = Self::new();
        for ad in file.advertisements {
            store.register(ad)?;
        }
        Ok(store)
    }

    /// Persist all advertisements (creates `capability/` as needed).
    pub fn save(&self, root: impl AsRef<Path>) -> Result<(), ProtocolError> {
        let path = Self::path(&root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        }
        let mut advertisements = self.ads.clone();
        advertisements.sort_by(|a, b| a.advertisement_id.cmp(&b.advertisement_id));
        let file = CapabilityAdFile {
            schema: CAPABILITY_AD_STORE_SCHEMA.into(),
            advertisements,
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| ProtocolError::Schema(e.to_string()))?;
        fs::write(path, format!("{json}\n")).map_err(|e| ProtocolError::Schema(e.to_string()))
    }

    /// True if an advertisement_id is already stored.
    pub fn contains(&self, advertisement_id: &str) -> bool {
        self.ads
            .iter()
            .any(|a| a.advertisement_id == advertisement_id)
    }

    /// Register (or replace by advertisement_id) a local capability advertisement.
    ///
    /// Enforces B2-005 required fields and rejects Node-keyed providers.
    pub fn register(&mut self, ad: CapabilityAdvertisement) -> Result<(), ProtocolError> {
        validate_advertisement(&ad)?;
        if let Some(idx) = self
            .ads
            .iter()
            .position(|a| a.advertisement_id == ad.advertisement_id)
        {
            self.ads[idx] = ad;
        } else {
            self.ads.push(ad);
        }
        Ok(())
    }

    /// List all advertisements (sorted by advertisement_id).
    pub fn list_all(&self) -> Vec<&CapabilityAdvertisement> {
        let mut ads: Vec<_> = self.ads.iter().collect();
        ads.sort_by(|a, b| a.advertisement_id.cmp(&b.advertisement_id));
        ads
    }

    /// Helper to build a signed local advertisement (B2-005 fields present).
    pub fn local_advertisement(
        advertisement_id: &str,
        capability_type: &str,
        provider_csu: &str,
    ) -> Result<CapabilityAdvertisement, ProtocolError> {
        Ok(CapabilityAdvertisement {
            advertisement_id: advertisement_id.into(),
            provider_csu: AiraRef::parse(provider_csu)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            capability: CapabilityAdBody {
                capability_type: capability_type.into(),
                schema_version: "0.1".into(),
                constraints: serde_json::json!({}),
                scope: ScopeDescriptor::local("capability-ad"),
                cost_model_ref: None,
                evidence_refs: vec![],
                confidence: Some(1.0),
            },
            policy_refs: vec![AiraRef::parse("aira:policy:default")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            validity_window: ValidityWindow {
                from: mvp_timestamp(),
                to: None,
            },
            signature: local_signature(),
        })
    }
}

/// Validate B2-005 MUST fields + anti-node provider rule.
fn validate_advertisement(ad: &CapabilityAdvertisement) -> Result<(), ProtocolError> {
    if ad.advertisement_id.trim().is_empty() {
        return Err(ProtocolError::Schema(
            "advertisement_id must be non-empty".into(),
        ));
    }
    if ad.provider_csu.as_str().contains(":node:") {
        return Err(ProtocolError::Schema(
            "capability advertisement must use provider CSU, not Node".into(),
        ));
    }
    if ad.capability.capability_type.trim().is_empty() {
        return Err(ProtocolError::Schema(
            "capability_type must be non-empty".into(),
        ));
    }
    if ad.capability.scope.scope_type.trim().is_empty() {
        return Err(ProtocolError::Schema("scope must be present".into()));
    }
    if ad.policy_refs.is_empty() {
        return Err(ProtocolError::Schema(
            "policy_refs must be non-empty (B2-005)".into(),
        ));
    }
    if ad.signature.signature_value.trim().is_empty() {
        return Err(ProtocolError::InvalidSignature);
    }
    Ok(())
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
        p.push(format!("aira-capability-ad-{nanos}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn capability_ad_persist_roundtrip() {
        let root = temp_root();
        let mut store = CapabilityAdvertisementStore::new();
        let ad = CapabilityAdvertisementStore::local_advertisement(
            "aira:capability-ad:local:execution-basic",
            "local.execution-basic",
            "aira:csu:execution-basic",
        )
        .unwrap();
        store.register(ad).unwrap();
        store.save(&root).unwrap();
        assert!(CapabilityAdvertisementStore::path(&root).exists());

        let loaded = CapabilityAdvertisementStore::load(&root).unwrap();
        assert!(loaded.contains("aira:capability-ad:local:execution-basic"));
        assert_eq!(loaded.list_all().len(), 1);
        let got = loaded.list_all()[0];
        assert_eq!(got.capability.capability_type, "local.execution-basic");
        assert!(!got.provider_csu.as_str().contains(":node:"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn capability_ad_load_empty_when_missing() {
        let root = temp_root();
        let loaded = CapabilityAdvertisementStore::load(&root).unwrap();
        assert!(loaded.list_all().is_empty());
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn capability_ad_rejects_node_provider() {
        let mut store = CapabilityAdvertisementStore::new();
        let mut ad = CapabilityAdvertisementStore::local_advertisement(
            "aira:capability-ad:bad",
            "local.bad",
            "aira:csu:execution-basic",
        )
        .unwrap();
        ad.provider_csu = AiraRef::parse("aira:node:local").unwrap();
        assert!(store.register(ad).is_err());
    }

    #[test]
    fn capability_ad_rejects_empty_signature() {
        let mut store = CapabilityAdvertisementStore::new();
        let mut ad = CapabilityAdvertisementStore::local_advertisement(
            "aira:capability-ad:unsigned",
            "local.unsigned",
            "aira:csu:execution-basic",
        )
        .unwrap();
        ad.signature.signature_value.clear();
        assert!(matches!(
            store.register(ad),
            Err(ProtocolError::InvalidSignature)
        ));
    }
}
