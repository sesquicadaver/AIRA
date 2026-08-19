//! Canonical JSON hashing and Ed25519 over the descriptor content hash (QUEUE #39).
//!
//! Schema Pack §2.2 / Book IV §20: UTF-8 Canonical JSON (sorted keys, no
//! insignificant whitespace) → SHA-256 → Ed25519 over `hash.as_str()` bytes.
//!
//! Production Event (#40), Artifact (#41), and Object (#42) verify paths are switched.
//! CSU remains unwired until #43.

use serde_json::{Map, Value};

use crate::crypto::{signature_for, verify_ed25519, CryptoError};
use crate::types::{AiraRef, ContentHash, Signature};

const SIGNATURE_KEY: &str = "signature";

/// Recursively sort object keys. Arrays keep order. Scalars unchanged.
pub fn canonicalize_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let mut out = Map::new();
            for k in keys {
                out.insert(k.clone(), canonicalize_value(&map[k]));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_value).collect()),
        other => other.clone(),
    }
}

/// Compact UTF-8 JSON with deterministic key order and no extra whitespace.
pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, CryptoError> {
    serde_json::to_vec(&canonicalize_value(value)).map_err(|e| CryptoError::Io(e.to_string()))
}

/// Clone `value` without a top-level `"signature"` member (nested signatures stay).
pub fn strip_top_level_signature(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = map.clone();
            out.remove(SIGNATURE_KEY);
            Value::Object(out)
        }
        other => other.clone(),
    }
}

/// SHA-256 of canonical JSON of the descriptor **without** top-level signature.
pub fn descriptor_signing_hash(value: &Value) -> Result<ContentHash, CryptoError> {
    let stripped = strip_top_level_signature(value);
    let bytes = canonical_json_bytes(&stripped)?;
    Ok(ContentHash::sha256_bytes(&bytes))
}

/// Bytes passed to Ed25519: the `sha256:…` hash string (same shape as today's payload_hash).
pub fn descriptor_signing_message(value: &Value) -> Result<Vec<u8>, CryptoError> {
    Ok(descriptor_signing_hash(value)?.as_str().as_bytes().to_vec())
}

/// Sign the canonical descriptor hash with the process keyring (`signature_for`).
pub fn sign_canonical_descriptor(
    key_ref: &AiraRef,
    value: &Value,
) -> Result<Signature, CryptoError> {
    let msg = descriptor_signing_message(value)?;
    signature_for(key_ref, &msg)
}

/// Verify `signature` over the canonical descriptor hash (no LOCAL_TEST domain fallback).
pub fn verify_canonical_descriptor(
    signature: &Signature,
    value: &Value,
) -> Result<(), CryptoError> {
    let msg = descriptor_signing_message(value)?;
    verify_ed25519(signature, &msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{local_test_signature, verify_ed25519, LOCAL_TEST_KEY_REF};
    use serde_json::json;

    #[test]
    fn key_order_and_whitespace_do_not_change_hash() {
        let a = json!({"b": 1, "a": {"z": true, "y": false}});
        let b = json!({"a": {"y": false, "z": true}, "b": 1});
        assert_eq!(
            descriptor_signing_hash(&a).unwrap(),
            descriptor_signing_hash(&b).unwrap()
        );
        let compact = canonical_json_bytes(&a).unwrap();
        assert!(!compact.contains(&b' '));
        assert!(!compact.contains(&b'\n'));
    }

    #[test]
    fn top_level_signature_is_stripped_from_hash() {
        let body = json!({"event_type": "ProblemSubmitted", "payload_hash": "sha256:ab"});
        let mut with_sig = body.clone();
        with_sig
            .as_object_mut()
            .unwrap()
            .insert("signature".into(), json!({"algorithm": "ed25519"}));
        assert_eq!(
            descriptor_signing_hash(&body).unwrap(),
            descriptor_signing_hash(&with_sig).unwrap()
        );
    }

    #[test]
    fn nested_signature_field_is_not_stripped() {
        let a = json!({"meta": {"signature": "keep"}});
        let b = json!({"meta": {"signature": "other"}});
        assert_ne!(
            descriptor_signing_hash(&a).unwrap().as_str(),
            descriptor_signing_hash(&b).unwrap().as_str()
        );
    }

    #[test]
    fn field_mutation_changes_hash() {
        let a = json!({"event_type": "ProblemSubmitted", "causal_refs": []});
        let b = json!({"event_type": "ResultPublished", "causal_refs": []});
        assert_ne!(
            descriptor_signing_hash(&a).unwrap(),
            descriptor_signing_hash(&b).unwrap()
        );
    }

    #[test]
    fn sign_verify_roundtrip_and_reject_mutation() {
        let key = AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap();
        let desc = json!({
            "event_type": "ProblemSubmitted",
            "causal_refs": ["aira:event:01"],
            "payload_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        });
        let sig = sign_canonical_descriptor(&key, &desc).unwrap();
        verify_canonical_descriptor(&sig, &desc).unwrap();
        let mut mutated = desc.clone();
        mutated["event_type"] = json!("CapsuleFailed");
        assert!(verify_canonical_descriptor(&sig, &mutated).is_err());
    }

    #[test]
    fn helper_does_not_accept_payload_hash_only_message() {
        let desc = json!({"payload_hash": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"});
        let sig =
            sign_canonical_descriptor(&AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap(), &desc).unwrap();
        let payload_only = desc["payload_hash"].as_str().unwrap().as_bytes();
        assert!(verify_ed25519(&sig, payload_only).is_err());
        verify_canonical_descriptor(&sig, &desc).unwrap();
    }

    #[test]
    fn existing_payload_hash_verify_path_still_independent() {
        let msg = b"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
        let sig = local_test_signature(msg);
        verify_ed25519(&sig, msg).unwrap();
        let desc = json!({"payload_hash": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"});
        assert!(verify_canonical_descriptor(&sig, &desc).is_err());
    }
}
