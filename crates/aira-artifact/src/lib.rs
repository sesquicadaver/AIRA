//! AIRA artifact store (Issue Set Epic 4 / #27–#29).
//!
//! Content-addressed, immutable artifacts with supersession metadata.

mod descriptor;
mod store;

pub use descriptor::{ArtifactDescriptor, ArtifactType};
pub use store::{ArtifactError, ArtifactStore, CasArtifactStore, PublishResult, SupersessionMeta};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ContentHash, Signature, Timestamp};

    fn sample_sig() -> Signature {
        Signature {
            algorithm: "ed25519".into(),
            key_ref: AiraRef::parse("aira:identity:local-test").unwrap(),
            signature_value: "TESTSIG".into(),
        }
    }

    fn descriptor_for(payload: &[u8], artifact_id: &str) -> ArtifactDescriptor {
        let hash = ContentHash::sha256_bytes(payload);
        ArtifactDescriptor {
            artifact_id: AiraRef::parse(artifact_id).unwrap(),
            artifact_type: ArtifactType::EvidenceArtifact,
            schema_version: "0.1".into(),
            content_hash: hash.clone(),
            content_ref: format!("cas://{}", hash.as_str()),
            producer_identity: AiraRef::parse("aira:identity:local-test").unwrap(),
            provenance_refs: vec![AiraRef::parse("aira:event:01E1").unwrap()],
            dependency_refs: vec![],
            policy_refs: vec![AiraRef::parse("aira:policy:default").unwrap()],
            signature: sample_sig(),
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
        }
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn descriptor_schema_valid() {
        let d = descriptor_for(b"hello", "aira:artifact:sha256:test1");
        // artifact_id pattern requires aira:kind:id — use valid ref
        let d = ArtifactDescriptor {
            artifact_id: AiraRef::parse(
                "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            )
            .unwrap(),
            ..d
        };
        let v = serde_json::to_value(&d).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:artifact:artifact-descriptor:0.1", &v)
            .unwrap();
    }

    #[test]
    fn cas_publish_resolve_and_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let payload = b"calculate-2-plus-2";
        let desc = descriptor_for(
            payload,
            "aira:artifact:sha256_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        );
        let published = store.publish(desc.clone(), payload).unwrap();
        assert_eq!(published.descriptor.content_hash, desc.content_hash);

        let (loaded, bytes) = store.resolve(&desc.artifact_id).unwrap();
        assert_eq!(loaded, desc);
        assert_eq!(bytes, payload);

        // hash mismatch rejected
        let bad = ArtifactDescriptor {
            content_hash: ContentHash::parse(
                "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            )
            .unwrap(),
            ..desc.clone()
        };
        let err = store.publish(bad, payload).unwrap_err();
        assert!(matches!(err, ArtifactError::HashMismatch { .. }));
    }

    #[test]
    fn mutation_fails_supersession_keeps_old() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let payload = b"v1";
        let id1 =
            "aira:artifact:sha256_1111111111111111111111111111111111111111111111111111111111111111";
        let d1 = descriptor_for(payload, id1);
        store.publish(d1.clone(), payload).unwrap();

        let err = store
            .replace_payload(&d1.artifact_id, b"mutated")
            .unwrap_err();
        assert!(matches!(err, ArtifactError::Immutable { .. }));

        let payload2 = b"v2";
        let id2 =
            "aira:artifact:sha256_2222222222222222222222222222222222222222222222222222222222222222";
        let d2 = descriptor_for(payload2, id2);
        let super_meta = store
            .supersede(&d1.artifact_id, d2.clone(), payload2)
            .unwrap();
        assert_eq!(super_meta.previous, d1.artifact_id);
        assert_eq!(super_meta.current, d2.artifact_id);

        let (old, old_bytes) = store.resolve(&d1.artifact_id).unwrap();
        assert_eq!(old, d1);
        assert_eq!(old_bytes, b"v1");
    }
}
