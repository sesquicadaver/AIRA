//! Local Settlement receipt store (Book II §15; QUEUE #173).
//!
//! Append-only JSONL under `settlement/receipts.jsonl`. Not a blockchain ledger.
//! Verify-on-read re-checks canonical Ed25519 over the receipt body (provider-bound).

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use aira_object::{
    sign_canonical_descriptor, verify_canonical_descriptor, verify_producer_signature_binding,
    AiraRef, Signature, Timestamp, LOCAL_TEST_KEY_REF,
};
use serde::{Deserialize, Serialize};

use crate::envelope::{mvp_timestamp, ProtocolError};

/// On-disk marker for the settlement receipts JSONL store.
pub const SETTLEMENT_RECEIPTS_STORE_SCHEMA: &str = "aira:settlement:receipts-jsonl:v1";

/// Relative path under a node root for the append-only receipts log.
pub const SETTLEMENT_RECEIPTS_REL: &str = "settlement/receipts.jsonl";

/// Contribution accounting fields (Book II §15.2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContributionDescriptor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub method: String,
}

/// Settlement Receipt (schema `aira:schema:settlement:receipt:0.1` + `privacy_class`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettlementReceipt {
    pub receipt_id: AiraRef,
    pub execution_or_artifact_ref: AiraRef,
    pub provider_identity: AiraRef,
    pub consumer_identity: AiraRef,
    pub capability_refs: Vec<AiraRef>,
    pub contribution_descriptor: ContributionDescriptor,
    pub cost_descriptor_ref: AiraRef,
    #[serde(default)]
    pub verification_refs: Vec<AiraRef>,
    pub policy_refs: Vec<AiraRef>,
    pub privacy_class: String,
    pub created_at: Timestamp,
    pub signature: Signature,
}

impl SettlementReceipt {
    /// Sign over canonical JSON without the top-level `signature` (provider key_ref).
    pub fn attach_canonical_signature(mut self) -> Result<Self, ProtocolError> {
        let v = serde_json::to_value(&self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        self.signature = sign_canonical_descriptor(&self.provider_identity, &v)
            .map_err(|_| ProtocolError::InvalidSignature)?;
        Ok(self)
    }

    /// Verify-on-read: provider binding + canonical Ed25519 (no LOCAL_TEST domain fallback).
    pub fn verify_canonical(&self) -> Result<(), ProtocolError> {
        verify_producer_signature_binding(&self.provider_identity, &self.signature)
            .map_err(|_| ProtocolError::InvalidSignature)?;
        let v = serde_json::to_value(self).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        verify_canonical_descriptor(&self.signature, &v)
            .map_err(|_| ProtocolError::InvalidSignature)
    }
}

/// Append-only local settlement receipt store (JSONL).
#[derive(Debug)]
pub struct SettlementReceiptStore {
    path: PathBuf,
    receipts: Vec<SettlementReceipt>,
}

impl SettlementReceiptStore {
    /// Path to durable JSONL for a node root.
    pub fn path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join(SETTLEMENT_RECEIPTS_REL)
    }

    /// Schema sidecar next to the JSONL (documents store format).
    pub fn schema_path(root: impl AsRef<Path>) -> PathBuf {
        root.as_ref().join("settlement").join("STORE_SCHEMA")
    }

    /// Open existing log (verify-on-read each line) or create empty store.
    pub fn open_or_create(root: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let path = Self::path(&root);
        if path.exists() {
            Self::open(root)
        } else {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| ProtocolError::Storage(e.to_string()))?;
            }
            fs::write(
                Self::schema_path(&root),
                format!("{SETTLEMENT_RECEIPTS_STORE_SCHEMA}\n"),
            )
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
            fs::write(&path, "").map_err(|e| ProtocolError::Storage(e.to_string()))?;
            Ok(Self {
                path,
                receipts: Vec::new(),
            })
        }
    }

    /// Open and verify every receipt line (fail closed on tamper / bad signature).
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ProtocolError> {
        let path = Self::path(&root);
        let schema_path = Self::schema_path(&root);
        if schema_path.exists() {
            let tag = fs::read_to_string(&schema_path)
                .map_err(|e| ProtocolError::Storage(e.to_string()))?;
            let tag = tag.trim();
            if tag != SETTLEMENT_RECEIPTS_STORE_SCHEMA {
                return Err(ProtocolError::Schema(format!(
                    "settlement store schema mismatch: {tag}"
                )));
            }
        }
        let file = fs::File::open(&path).map_err(|e| ProtocolError::Storage(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut receipts = Vec::new();
        for (idx, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| ProtocolError::Storage(e.to_string()))?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let receipt: SettlementReceipt = serde_json::from_str(line)
                .map_err(|e| ProtocolError::Schema(format!("receipt line {}: {e}", idx + 1)))?;
            admit_receipt(&receipt)?;
            receipts.push(receipt);
        }
        Ok(Self { path, receipts })
    }

    pub fn path_ref(&self) -> &Path {
        &self.path
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    /// Append one verified receipt (idempotent if identical body for same `receipt_id`).
    pub fn append(&mut self, receipt: SettlementReceipt) -> Result<(), ProtocolError> {
        admit_receipt(&receipt)?;
        if let Some(existing) = self
            .receipts
            .iter()
            .find(|r| r.receipt_id == receipt.receipt_id)
        {
            if existing == &receipt {
                return Ok(());
            }
            return Err(ProtocolError::Duplicate(receipt.receipt_id));
        }
        let line =
            serde_json::to_string(&receipt).map_err(|e| ProtocolError::Schema(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| ProtocolError::Storage(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| ProtocolError::Storage(e.to_string()))?;
        self.receipts.push(receipt);
        Ok(())
    }

    /// Lookup by `receipt_id` with verify-on-read.
    pub fn get(&self, receipt_id: &AiraRef) -> Result<Option<SettlementReceipt>, ProtocolError> {
        match self.receipts.iter().find(|r| &r.receipt_id == receipt_id) {
            None => Ok(None),
            Some(r) => {
                admit_receipt(r)?;
                Ok(Some(r.clone()))
            }
        }
    }

    /// All receipts in append order (each re-verified).
    pub fn list_all(&self) -> Result<Vec<SettlementReceipt>, ProtocolError> {
        let mut out = Vec::with_capacity(self.receipts.len());
        for r in &self.receipts {
            admit_receipt(r)?;
            out.push(r.clone());
        }
        Ok(out)
    }

    /// Build a locally signed sample receipt (provider = `aira:identity:local-test`).
    pub fn local_receipt(receipt_id: &str) -> Result<SettlementReceipt, ProtocolError> {
        let receipt = SettlementReceipt {
            receipt_id: AiraRef::parse(receipt_id)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            execution_or_artifact_ref: AiraRef::parse("aira:artifact:verified:local")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            provider_identity: AiraRef::parse(LOCAL_TEST_KEY_REF)
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            consumer_identity: AiraRef::parse("aira:identity:local-consumer")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            capability_refs: vec![AiraRef::parse("aira:capability:math.eval.safe")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            contribution_descriptor: ContributionDescriptor {
                amount: Some(0.0),
                unit: Some("local-ops".into()),
                method: "own-resource".into(),
            },
            cost_descriptor_ref: AiraRef::parse("aira:artifact:cost:local")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?,
            verification_refs: vec![AiraRef::parse("aira:artifact:evidence:local")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            policy_refs: vec![AiraRef::parse("aira:policy:default")
                .map_err(|e| ProtocolError::Schema(e.to_string()))?],
            privacy_class: "audit-safe".into(),
            created_at: mvp_timestamp(),
            signature: crate::envelope::local_signature(),
        };
        receipt.attach_canonical_signature()
    }
}

fn admit_receipt(receipt: &SettlementReceipt) -> Result<(), ProtocolError> {
    if receipt.privacy_class.trim().is_empty() {
        return Err(ProtocolError::Schema(
            "privacy_class must be non-empty (PRIV-001 / Book II §15.3)".into(),
        ));
    }
    if receipt.capability_refs.is_empty() {
        return Err(ProtocolError::Schema(
            "capability_refs must be non-empty".into(),
        ));
    }
    if receipt.policy_refs.is_empty() {
        return Err(ProtocolError::Schema(
            "policy_refs must be non-empty".into(),
        ));
    }
    if receipt.contribution_descriptor.method.trim().is_empty() {
        return Err(ProtocolError::Schema(
            "contribution_descriptor.method must be non-empty".into(),
        ));
    }
    if receipt.signature.signature_value.trim().is_empty() {
        return Err(ProtocolError::InvalidSignature);
    }
    receipt.verify_canonical()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_receipt_store_append_roundtrip_and_verify_on_read() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut store = SettlementReceiptStore::open_or_create(root).unwrap();
        assert!(store.is_empty());
        assert!(SettlementReceiptStore::path(root).exists());
        assert_eq!(
            fs::read_to_string(SettlementReceiptStore::schema_path(root))
                .unwrap()
                .trim(),
            SETTLEMENT_RECEIPTS_STORE_SCHEMA
        );

        let r1 = SettlementReceiptStore::local_receipt("aira:settlement:receipt:s1").unwrap();
        store.append(r1.clone()).unwrap();
        store
            .append(SettlementReceiptStore::local_receipt("aira:settlement:receipt:s2").unwrap())
            .unwrap();
        assert_eq!(store.len(), 2);
        // Idempotent identical re-append.
        store.append(r1.clone()).unwrap();
        assert_eq!(store.len(), 2);

        drop(store);
        let reopened = SettlementReceiptStore::open(root).unwrap();
        assert_eq!(reopened.len(), 2);
        let got = reopened
            .get(&AiraRef::parse("aira:settlement:receipt:s1").unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(got.privacy_class, "audit-safe");
        assert_eq!(got.contribution_descriptor.method, "own-resource");
        got.verify_canonical().unwrap();
    }

    #[test]
    fn settlement_receipt_store_rejects_tampered_line_on_open() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut store = SettlementReceiptStore::open_or_create(root).unwrap();
        store
            .append(SettlementReceiptStore::local_receipt("aira:settlement:receipt:t1").unwrap())
            .unwrap();
        drop(store);

        let path = SettlementReceiptStore::path(root);
        let mut raw = fs::read_to_string(&path).unwrap();
        raw = raw.replace("audit-safe", "tampered-class");
        fs::write(&path, raw).unwrap();

        let err = SettlementReceiptStore::open(root).unwrap_err();
        assert!(matches!(err, ProtocolError::InvalidSignature));
    }

    #[test]
    fn settlement_receipt_store_rejects_empty_privacy_class() {
        let mut empty_priv =
            SettlementReceiptStore::local_receipt("aira:settlement:receipt:empty-priv").unwrap();
        empty_priv.privacy_class = "   ".into();
        empty_priv = empty_priv.attach_canonical_signature().unwrap();
        assert!(matches!(
            admit_receipt(&empty_priv),
            Err(ProtocolError::Schema(_))
        ));

        let mut unsigned =
            SettlementReceiptStore::local_receipt("aira:settlement:receipt:unsigned").unwrap();
        unsigned.signature.signature_value.clear();
        assert!(matches!(
            admit_receipt(&unsigned),
            Err(ProtocolError::InvalidSignature)
        ));
    }

    #[test]
    fn settlement_receipt_store_rejects_duplicate_different_body() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = SettlementReceiptStore::open_or_create(dir.path()).unwrap();
        let a = SettlementReceiptStore::local_receipt("aira:settlement:receipt:dup").unwrap();
        store.append(a).unwrap();
        let mut b = SettlementReceiptStore::local_receipt("aira:settlement:receipt:dup").unwrap();
        b.contribution_descriptor.method = "other-method".into();
        b = b.attach_canonical_signature().unwrap();
        assert!(matches!(store.append(b), Err(ProtocolError::Duplicate(_))));
    }
}
