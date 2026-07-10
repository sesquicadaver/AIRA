//! AIRA core object model (Issue Set Epic 3 / #22–#24).
//!
//! Provides typed references, hashes, signatures, opaque handles, and
//! `ObjectDescriptor` aligned with Schema Pack core schemas.

mod descriptor;
mod handle;
mod types;

pub use descriptor::{ObjectDescriptor, ObjectType};
pub use handle::Handle;
pub use types::{AiraRef, ContentHash, Signature, Timestamp};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn aira_ref_roundtrip() {
        let r = AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap();
        let s = serde_json::to_string(&r).unwrap();
        let back: AiraRef = serde_json::from_str(&s).unwrap();
        assert_eq!(r, back);
        assert_eq!(r.as_str(), "aira:problem:01TESTPROBLEM");
    }

    #[test]
    fn aira_ref_rejects_invalid() {
        assert!(AiraRef::parse("not-a-ref").is_err());
        assert!(AiraRef::parse("aira:BAD:x").is_err());
    }

    #[test]
    fn hash_roundtrip() {
        let h = ContentHash::parse(
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap();
        let s = serde_json::to_string(&h).unwrap();
        let back: ContentHash = serde_json::from_str(&s).unwrap();
        assert_eq!(h, back);
    }

    #[test]
    fn signature_roundtrip() {
        let sig = Signature {
            algorithm: "ed25519".into(),
            key_ref: AiraRef::parse("aira:identity:local-test").unwrap(),
            signature_value: "TESTSIG".into(),
        };
        let v = serde_json::to_value(&sig).unwrap();
        let back: Signature = serde_json::from_value(v).unwrap();
        assert_eq!(sig, back);
    }

    #[test]
    fn handle_is_opaque() {
        let object_ref = AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap();
        let h = Handle::new(object_ref.clone(), 42);
        let dbg = format!("{h:?}");
        assert!(!dbg.contains("sqlite"));
        assert!(!dbg.contains("/"));
        assert!(!dbg.contains("path"));
        // Public API exposes only the logical ref, not storage internals.
        assert_eq!(h.object_ref(), &object_ref);
        assert!(h.storage_token_for_tests() > 0);
    }

    #[test]
    fn object_descriptor_schema_valid() {
        let desc = ObjectDescriptor::example_problem();
        let value = serde_json::to_value(&desc).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:core:object-descriptor:0.1", &value)
            .unwrap();
    }

    #[test]
    fn forbidden_object_types_rejected() {
        for bad in ["GPU", "Node", "Driver", "Scheduler", "Blockchain", "Wallet"] {
            let v = json!({
                "object_id": "aira:problem:01TESTPROBLEM",
                "object_type": bad,
                "schema_version": "0.1",
                "created_at": "2026-07-10T12:00:00Z",
                "producer_identity": "aira:identity:local-test",
                "policy_refs": [],
                "provenance_refs": [],
                "content_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "signature": {
                    "algorithm": "ed25519",
                    "key_ref": "aira:identity:local-test",
                    "signature_value": "TESTSIG"
                }
            });
            let err = serde_json::from_value::<ObjectDescriptor>(v).unwrap_err();
            assert!(
                err.to_string().contains(bad) || err.to_string().contains("object_type"),
                "expected reject for {bad}, got {err}"
            );
        }
    }
}
