//! AIRA artifact store (Issue Set Epic 4 / #27–#29).
//!
//! Content-addressed, immutable artifacts with supersession metadata.

mod descriptor;
mod store;

pub use descriptor::{ArtifactDescriptor, ArtifactType};
pub use store::{
    is_private_artifact, ArtifactError, ArtifactStore, CasArtifactStore, PublishResult,
    SupersessionMeta, PRIVATE_ARTIFACT_POLICY,
};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_object::{AiraRef, ContentHash, Timestamp};

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
            signature: aira_object::local_test_signature(hash.as_str().as_bytes()),
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

        // hash mismatch rejected (signature binds the claimed content_hash)
        let bad_hash = ContentHash::parse(
            "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        )
        .unwrap();
        let bad = ArtifactDescriptor {
            content_hash: bad_hash.clone(),
            signature: aira_object::local_test_signature(bad_hash.as_str().as_bytes()),
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

    #[test]
    fn cas_index_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let payload = b"persist-me";
        let id =
            "aira:artifact:sha256_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        {
            let mut store = CasArtifactStore::open(dir.path()).unwrap();
            store.publish(descriptor_for(payload, id), payload).unwrap();
        }
        let store = CasArtifactStore::open(dir.path()).unwrap();
        let (desc, bytes) = store.resolve(&AiraRef::parse(id).unwrap()).unwrap();
        assert_eq!(bytes, payload);
        assert_eq!(desc.artifact_id.as_str(), id);
    }

    #[test]
    fn unsigned_artifact_rejected_and_private_denied() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = CasArtifactStore::open(dir.path()).unwrap();
        let payload = b"private-bytes";
        let mut unsigned = descriptor_for(
            payload,
            "aira:artifact:sha256_3333333333333333333333333333333333333333333333333333333333333333",
        );
        unsigned.signature.signature_value.clear();
        let err = store.publish(unsigned, payload).unwrap_err();
        assert!(matches!(err, ArtifactError::Unsigned(_)));

        let mut private = descriptor_for(
            payload,
            "aira:artifact:sha256_4444444444444444444444444444444444444444444444444444444444444444",
        );
        private.policy_refs = vec![AiraRef::parse(PRIVATE_ARTIFACT_POLICY).unwrap()];
        store.publish(private.clone(), payload).unwrap();
        let denied = store.resolve(&private.artifact_id).unwrap_err();
        assert!(matches!(denied, ArtifactError::AccessDenied(_)));
        let (got, bytes) = store
            .resolve_with_access(&private.artifact_id, true)
            .unwrap();
        assert_eq!(got.artifact_id, private.artifact_id);
        assert_eq!(bytes, payload);
        assert!(is_private_artifact(&got));
    }
}
