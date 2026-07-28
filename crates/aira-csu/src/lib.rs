//! AIRA CSU runtime (Issue Set Epic 5 / #35–#40).
//!
//! Manifest, registry, lifecycle, in-process trait, event dispatch, isolation baseline.

mod error;
mod lifecycle;
mod manifest;
mod registry;
mod runtime;
pub mod support;

pub use error::CsuError;
pub use lifecycle::CsuLifecycleState;
pub use manifest::{CapabilityDescriptor, CsuManifest, CsuSandbox, CsuType, SUPPORTED_ABI_VERSION};
pub use registry::{CsuRegistry, RegisteredCsu};
pub use runtime::{Csu, CsuExecutionContext, CsuHandlerError, CsuOutput, CsuRuntime};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_event::{EventDescriptor, EventType, MemoryEventLog};
    use aira_object::{AiraRef, ContentHash, Signature, Timestamp};
    use serde_json::json;
    use std::path::PathBuf;

    fn producer() -> AiraRef {
        AiraRef::parse("aira:identity:local-test").unwrap()
    }

    fn sig() -> Signature {
        aira_object::local_test_signature(aira_object::LOCAL_TEST_DOMAIN_MSG)
    }

    fn sample_manifest() -> CsuManifest {
        // Prefer programmatic signed manifest (Alpha.2 crypto) over static TESTSIG fixtures.
        support::basic_manifest(
            "aira:csu:execution.basic",
            "execution-basic",
            CsuType::Execution,
            &["CapsuleCreated"],
            &["CapsuleCompleted"],
        )
    }

    fn sample_event(event_type: EventType) -> EventDescriptor {
        let payload_hash = ContentHash::parse(
            "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        )
        .unwrap();
        EventDescriptor {
            event_id: AiraRef::parse("aira:event:01E1").unwrap(),
            event_type,
            schema_version: "0.1".into(),
            producer_identity: AiraRef::parse("aira:identity:local-test").unwrap(),
            causal_refs: vec![],
            object_refs: vec![],
            artifact_refs: vec![],
            policy_refs: vec![],
            payload_hash: payload_hash.clone(),
            payload_ref: None,
            created_at: Timestamp::parse("2026-07-10T12:00:00Z").unwrap(),
            signature: aira_object::local_test_signature(payload_hash.as_str().as_bytes()),
        }
    }

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct EchoCsu {
        manifest: CsuManifest,
        received: Arc<AtomicUsize>,
    }

    impl Csu for EchoCsu {
        fn manifest(&self) -> &CsuManifest {
            &self.manifest
        }

        fn on_event(
            &mut self,
            _event: &EventDescriptor,
            _ctx: &mut CsuExecutionContext<'_, '_>,
        ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
            self.received.fetch_add(1, Ordering::SeqCst);
            Ok(vec![])
        }
    }

    struct FailingCsu {
        manifest: CsuManifest,
    }

    impl Csu for FailingCsu {
        fn manifest(&self) -> &CsuManifest {
            &self.manifest
        }

        fn on_event(
            &mut self,
            _event: &EventDescriptor,
            _ctx: &mut CsuExecutionContext<'_, '_>,
        ) -> Result<Vec<CsuOutput>, CsuHandlerError> {
            Err(CsuHandlerError {
                message: "boom".into(),
            })
        }
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn manifest_schema_valid_and_unsigned_rejected() {
        let m = sample_manifest();
        let v = serde_json::to_value(&m).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:csu:manifest:0.1", &v).unwrap();

        let unsigned_path = root.join("fixtures/invalid/csu/manifest-unsigned.json");
        let unsigned: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(unsigned_path).unwrap()).unwrap();
        assert!(reg
            .validate("aira:schema:csu:manifest:0.1", &unsigned)
            .is_err());

        let mut bad = m.clone();
        bad.signature.signature_value.clear();
        assert!(matches!(
            bad.validate_for_registration(),
            Err(CsuError::UnsignedManifest(_))
        ));
    }

    #[test]
    fn registry_register_abi_and_list() {
        let mut reg = CsuRegistry::new();
        let m = sample_manifest();
        reg.register(m.clone(), None).unwrap();
        assert_eq!(reg.list().len(), 1);
        assert_eq!(reg.list()[0].manifest.csu_id, m.csu_id);

        let mut bad_abi = m;
        bad_abi.abi_version = "9.9".into();
        bad_abi.csu_id = AiraRef::parse("aira:csu:other.basic").unwrap();
        bad_abi.signature = aira_object::local_test_signature(bad_abi.csu_id.as_str().as_bytes());
        assert!(matches!(
            reg.register(bad_abi, None),
            Err(CsuError::UnsupportedAbi(_))
        ));
    }

    #[test]
    fn lifecycle_transitions_and_events() {
        let mut log = MemoryEventLog::new();
        let mut reg = CsuRegistry::new().with_event_identity(producer(), sig());
        let m = sample_manifest();
        let id = m.csu_id.clone();
        reg.register(m, Some(&mut log)).unwrap();
        assert!(matches!(
            reg.transition(&id, CsuLifecycleState::Active, None),
            Err(CsuError::InvalidTransition { .. })
        ));
        reg.activate(&id, Some(&mut log)).unwrap();
        assert_eq!(reg.get(&id).unwrap().state, CsuLifecycleState::Active);
        reg.suspend(&id, Some(&mut log)).unwrap();
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::CSURegistered));
        assert!(log
            .all()
            .iter()
            .any(|e| e.event_type == EventType::CSUSuspended));
    }

    #[test]
    fn dispatch_active_only_and_failure_event() {
        let mut log = MemoryEventLog::new();
        let mut rt = CsuRuntime::new(producer(), sig());
        let mut manifest = sample_manifest();
        manifest.event_subscriptions = vec![json!({"event_type": "ProblemSubmitted"})];
        let id = manifest.csu_id.clone();
        let received = Arc::new(AtomicUsize::new(0));
        rt.register_handler(
            Box::new(EchoCsu {
                manifest,
                received: received.clone(),
            }),
            Some(&mut log),
        )
        .unwrap();
        rt.activate(&id, Some(&mut log)).unwrap();

        let ev = sample_event(EventType::ProblemSubmitted);
        rt.dispatch(&ev, &mut log).unwrap();
        assert_eq!(received.load(Ordering::SeqCst), 1);

        // Suspend → no further delivery
        rt.suspend(&id, Some(&mut log)).unwrap();
        rt.dispatch(&ev, &mut log).unwrap();
        assert_eq!(received.load(Ordering::SeqCst), 1);

        // Failure path
        let mut rt2 = CsuRuntime::new(producer(), sig());
        let mut m2 = sample_manifest();
        m2.csu_id = AiraRef::parse("aira:csu:fail.basic").unwrap();
        m2.signature = aira_object::local_test_signature(m2.csu_id.as_str().as_bytes());
        m2.event_subscriptions = vec![json!({"event_type": "ProblemSubmitted"})];
        let id2 = m2.csu_id.clone();
        rt2.register_handler(Box::new(FailingCsu { manifest: m2 }), None)
            .unwrap();
        rt2.activate(&id2, None).unwrap();
        let err = rt2
            .dispatch(&sample_event(EventType::ProblemSubmitted), &mut log)
            .unwrap_err();
        assert!(matches!(err, CsuError::Dispatch(_)));
        let failed = log
            .all()
            .iter()
            .find(|e| e.event_type == EventType::CSUFailed)
            .expect("CSUFailed");
        assert_eq!(failed.producer_identity.as_str(), producer().as_str());
        assert_eq!(failed.signature.key_ref.as_str(), producer().as_str());
        assert_eq!(failed.payload_ref.as_deref(), Some("boom"));
    }

    #[test]
    fn emit_failed_and_lifecycle_use_publisher_identity() {
        use aira_object::{
            register_csu_tenant_signing, reset_primary_signer, set_primary_signer, signature_for,
            unregister_csu_tenant, verify_ed25519, LOCAL_TEST_KEY_REF,
        };
        use ed25519_dalek::SigningKey;

        let pub_sk = SigningKey::from_bytes(&[41u8; 32]);
        let pub_id = AiraRef::parse("aira:identity:csu-fail-publisher").unwrap();
        set_primary_signer(AiraRef::parse(LOCAL_TEST_KEY_REF).unwrap());

        let mut log = MemoryEventLog::new();
        let mut rt = CsuRuntime::new(producer(), sig());
        let mut m = sample_manifest();
        m.csu_id = AiraRef::parse("aira:csu:fail.publisher").unwrap();
        m.signature = aira_object::local_test_signature(m.csu_id.as_str().as_bytes());
        m.event_subscriptions = vec![json!({"event_type": "ProblemSubmitted"})];
        support::apply_publisher(&mut m, pub_id.clone());
        register_csu_tenant_signing(&m.csu_id, pub_id.clone(), pub_sk).unwrap();
        let id = m.csu_id.clone();

        rt.register_handler(Box::new(FailingCsu { manifest: m }), Some(&mut log))
            .unwrap();
        let registered = log
            .all()
            .iter()
            .find(|e| e.event_type == EventType::CSURegistered)
            .expect("CSURegistered");
        assert_eq!(registered.producer_identity.as_str(), pub_id.as_str());
        assert_eq!(registered.signature.key_ref.as_str(), pub_id.as_str());
        verify_ed25519(
            &registered.signature,
            registered.payload_hash.as_str().as_bytes(),
        )
        .unwrap();
        assert!(matches!(
            signature_for(&pub_id, b"x"),
            Err(aira_object::CryptoError::NoSigningKey(_))
        ));

        rt.activate(&id, Some(&mut log)).unwrap();
        let err = rt
            .dispatch(&sample_event(EventType::ProblemSubmitted), &mut log)
            .unwrap_err();
        assert!(matches!(err, CsuError::Dispatch(_)));
        let failed = log
            .all()
            .iter()
            .find(|e| e.event_type == EventType::CSUFailed)
            .expect("CSUFailed");
        assert_eq!(failed.producer_identity.as_str(), pub_id.as_str());
        assert_eq!(failed.signature.key_ref.as_str(), pub_id.as_str());
        assert_eq!(failed.payload_ref.as_deref(), Some("boom"));
        verify_ed25519(&failed.signature, failed.payload_hash.as_str().as_bytes()).unwrap();

        // Missing signing key → fail closed (no CSUFailed with wrong producer).
        let mut rt2 = CsuRuntime::new(producer(), sig());
        let mut m2 = sample_manifest();
        m2.csu_id = AiraRef::parse("aira:csu:fail.nosign").unwrap();
        m2.signature = aira_object::local_test_signature(m2.csu_id.as_str().as_bytes());
        m2.event_subscriptions = vec![json!({"event_type": "ProblemSubmitted"})];
        let missing = AiraRef::parse("aira:identity:no-signing-key").unwrap();
        support::apply_publisher(&mut m2, missing);
        let id2 = m2.csu_id.clone();
        // Lifecycle emit also fail-closed when events sink is provided.
        let mut log2 = MemoryEventLog::new();
        assert!(rt2
            .register_handler(Box::new(FailingCsu { manifest: m2 }), Some(&mut log2))
            .is_err());
        // Without lifecycle sink, register succeeds; dispatch emit_failed fails closed.
        let mut m3 = sample_manifest();
        m3.csu_id = AiraRef::parse("aira:csu:fail.nosign2").unwrap();
        m3.signature = aira_object::local_test_signature(m3.csu_id.as_str().as_bytes());
        m3.event_subscriptions = vec![json!({"event_type": "ProblemSubmitted"})];
        support::apply_publisher(
            &mut m3,
            AiraRef::parse("aira:identity:no-signing-key").unwrap(),
        );
        let id3 = m3.csu_id.clone();
        rt2.register_handler(Box::new(FailingCsu { manifest: m3 }), None)
            .unwrap();
        rt2.activate(&id3, None).unwrap();
        let before = log.all().len();
        let err2 = rt2
            .dispatch(&sample_event(EventType::ProblemSubmitted), &mut log)
            .unwrap_err();
        assert!(matches!(err2, CsuError::Dispatch(_)));
        assert_eq!(
            log.all()
                .iter()
                .filter(|e| e.event_type == EventType::CSUFailed
                    && e.object_refs.iter().any(|r| r.as_str() == id3.as_str()))
                .count(),
            0,
            "no CSUFailed when publisher cannot sign"
        );
        assert_eq!(log.all().len(), before);
        let _ = id2;

        unregister_csu_tenant(&id);
        unregister_csu_tenant(&AiraRef::parse("aira:csu:fail.nosign").unwrap());
        unregister_csu_tenant(&id3);
        reset_primary_signer();
    }

    #[test]
    fn isolation_baseline_denies_direct_mutation_and_peer_call() {
        let mut log = MemoryEventLog::new();
        let ctx = CsuExecutionContext::new(
            AiraRef::parse("aira:csu:execution.basic").unwrap(),
            &mut log,
            None,
            None,
        );
        assert!(matches!(
            ctx.mutate_core_object(&AiraRef::parse("aira:problem:01TESTPROBLEM").unwrap()),
            Err(CsuError::Isolation(_))
        ));
        assert!(matches!(
            ctx.mutate_artifact(&AiraRef::parse(
                "aira:artifact:sha256_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
            )
            .unwrap()),
            Err(CsuError::Isolation(_))
        ));
        assert!(matches!(
            ctx.call_csu(&AiraRef::parse("aira:csu:other.basic").unwrap()),
            Err(CsuError::Isolation(_))
        ));
    }

    #[test]
    fn registry_save_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path: PathBuf = dir.path().join("registry.json");
        let mut reg = CsuRegistry::new();
        reg.register(sample_manifest(), None).unwrap();
        reg.save(&path).unwrap();
        let loaded = CsuRegistry::load(&path).unwrap();
        assert_eq!(loaded.list().len(), 1);
    }
}
