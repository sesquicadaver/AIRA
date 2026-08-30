//! AIRA local operational flow (Issue Set Epic 7 / #47–#56 + Epic 8 local node).
//!
//! Wires Problem submit → basic CSU pipeline → Verified Result / Evidence / Epistemic assessment.
//! Epic 8 adds `.aira` layout persistence via [`local`].
//!
//! [`OperationalPlane`] is a **C1 reference/demo** (Analyze-86): not a production
//! event runtime, scheduler, distributed runtime, or federation runtime.
//! Operator-facing status: `docs/operational-plane.md`.

mod local;
mod plane;

pub use local::{
    init_node, load_config, node_config_present, open_node_sqlite_object_store,
    read_event_log_resilient, EventLogFile, EventLogReadOutcome, LocalSession, NodeConfig,
    NodePaths, ProblemRecord, DEFAULT_AIRA_ROOT, EVENT_LOG_CORRUPT_BACKUP,
};
pub use plane::{FlowError, OperationalPlane, SubmitOutcome};

/// Crate version string.
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aira_artifact::{ArtifactStore, ArtifactType};
    use aira_core::ObjectStore;
    use aira_csu::support::{json_bytes, make_artifact, make_event};
    use aira_event::EventType;
    use aira_object::AiraRef;
    use serde_json::json;
    use std::path::PathBuf;
    use std::sync::{Mutex, OnceLock};

    /// Process-wide CSU tenant map / primary signer must not race across parallel tests.
    fn isolated_flow() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let g = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        aira_object::reset_csu_tenants();
        aira_object::reset_primary_signer();
        g
    }

    #[test]
    fn version_is_semver_like() {
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn submit_creates_problem_and_event_schema_valid() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        assert!(matches!(out, SubmitOutcome::Completed { .. }));
        let problem = plane.problem_ref().unwrap();
        let desc = plane.objects().get_by_object_id(problem).unwrap().unwrap();
        let v = serde_json::to_value(&desc).unwrap();
        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:core:object-descriptor:0.1", &v)
            .unwrap();
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ProblemSubmitted));
    }

    #[test]
    fn calculate_two_plus_two_demo() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        let SubmitOutcome::Completed {
            verified_artifact_id,
            result,
            ..
        } = out
        else {
            panic!("expected completed flow, got {out:?}");
        };
        assert_eq!(result["result"], json!(4.0));
        assert_eq!(result["verification_status"], json!("VERIFIED"));
        assert_eq!(result["confidence"], json!(1.0));
        assert!(result.get("evidence_refs").is_some());
        assert!(result.get("provenance_refs").is_some());

        let (_d, bytes) = plane.artifacts().resolve(&verified_artifact_id).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["result"], json!(4.0));

        for required in [
            EventType::ProblemSubmitted,
            EventType::ContextResolved,
            EventType::ReductionCompleted,
            EventType::CapsuleCreated,
            EventType::CapsuleCompleted,
            EventType::VerificationCompleted,
            EventType::ResultPublished,
        ] {
            assert!(
                plane.events().iter().any(|e| e.event_type == required),
                "missing {required:?}"
            );
        }
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ArtifactPublished
                || e.event_type == EventType::FailureEvidenceCreated
                || plane.artifacts().resolve(&verified_artifact_id).is_ok()));
        // Evidence artifact exists for ResultPublished
        assert!(plane.has_evidence_for_results());
    }

    #[test]
    fn ready_solution_reuse_skips_execution() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let ready_payload = json_bytes(&json!({
            "result": 4.0,
            "verification_status": "VERIFIED",
            "confidence": 1.0,
            "scope": { "scope_type": "local", "description": "ready" },
            "evidence_refs": [],
            "provenance_refs": []
        }));
        let ready = make_artifact(
            "aira:artifact:ready42",
            ArtifactType::ReadySolutionArtifact,
            &ready_payload,
            vec![],
        );
        let ready_id = ready.artifact_id.clone();
        plane
            .artifacts_mut()
            .publish(ready, &ready_payload)
            .unwrap();
        plane.enable_ready_solution(ready_id).unwrap();

        let out = plane.submit_problem("Calculate 2 + 2").unwrap();
        assert!(matches!(out, SubmitOutcome::Completed { .. }));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::ResultPublished));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.payload_ref.as_deref() == Some("reuse:ready_solution")));
    }

    #[test]
    fn failure_to_evidence_demo() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let missing = AiraRef::parse("aira:artifact:missing99").unwrap();
        let ev = make_event(
            "aira:event:badcap1",
            EventType::CapsuleCreated,
            vec![AiraRef::parse("aira:problem:fail1").unwrap()],
            vec![missing],
            vec![],
            Some("math.eval.safe".into()),
        );
        plane.inject_and_drain(ev).unwrap();
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleFailed));
        assert!(plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::FailureEvidenceCreated));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::VerificationCompleted));
        assert!(!plane.has_verified_result_artifact());
    }

    #[test]
    fn normative_split_stub_does_not_autocollapse() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let out = plane
            .submit_problem("Choose either left-norm OR right-norm")
            .unwrap();
        let SubmitOutcome::NeedsHumanCollapse { field_artifact_id } = out else {
            panic!("expected NeedsHumanCollapse, got {out:?}");
        };
        let (desc, bytes) = plane.artifacts().resolve(&field_artifact_id).unwrap();
        assert_eq!(desc.artifact_type, ArtifactType::OperationalArtifact);
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["requires_human_collapse"], json!(true));
        assert!(!plane
            .events()
            .iter()
            .any(|e| e.event_type == EventType::CapsuleCompleted));
    }

    #[test]
    fn epistemic_assessment_roundtrip_via_plane_and_session() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let arts = dir.path().join("arts");
        let mut plane = OperationalPlane::open(&arts).unwrap();
        plane.submit_problem("Calculate 2 + 2").unwrap();
        let (aid, body) = plane
            .latest_epistemic_assessment()
            .expect("epistemic assessment after ResultPublished");
        assert_eq!(body["epistemic_status"], json!("Hypothesis"));
        assert!(body.get("evidence_refs").is_some());
        assert!(body.get("confidence").is_some());
        assert!(body.get("scope").is_some());
        assert!(body.get("counter_evidence_refs").is_some());
        assert!(body.get("revision_refs").is_some());

        let root = aira_schema::find_repo_root(env!("CARGO_MANIFEST_DIR")).unwrap();
        let reg = aira_schema::SchemaRegistry::load(root.join("schemas")).unwrap();
        reg.validate("aira:schema:epistemic:assessment:0.1", &body)
            .unwrap();

        // LocalSession / CLI path: submit + resolve assessment artifact bytes.
        let node = dir.path().join(".aira");
        init_node(&node).unwrap();
        let mut session = LocalSession::open(&node).unwrap();
        session.submit_problem("Calculate 2 + 2").unwrap();
        let (id, _) = session
            .plane()
            .latest_epistemic_assessment()
            .expect("session epistemic path");
        let (desc, bytes) = session.get_artifact(id.as_str()).unwrap();
        assert_eq!(desc["artifact_type"], json!("KnowledgeArtifact"));
        let again: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(again["epistemic_status"], json!("Hypothesis"));
        let _ = aid;
    }

    #[test]
    fn local_init_submit_status_and_artifact() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let cfg = load_config(&root).unwrap();
        assert_eq!(cfg.node.profile, "C1");
        assert!(root.join("db/aira.sqlite").exists());
        assert!(root.join("artifacts/sha256").exists());

        let mut session = LocalSession::open(&root).unwrap();
        let out = session.submit_problem("Calculate 2 + 2").unwrap();
        let SubmitOutcome::Completed {
            problem_id,
            verified_artifact_id,
            ..
        } = out
        else {
            panic!("expected completed");
        };

        let status = session.problem_status(problem_id.as_str()).unwrap();
        assert_eq!(status.status, "completed");
        let result = session.get_result(problem_id.as_str()).unwrap();
        assert_eq!(result["result"], json!(4.0));
        let (_desc, bytes) = session.get_artifact(verified_artifact_id.as_str()).unwrap();
        assert!(!bytes.is_empty());
        let tail = session.event_tail(50).unwrap();
        assert!(tail
            .iter()
            .any(|e| e.event_type == EventType::ProblemSubmitted));
    }

    #[test]
    fn local_session_submit_signs_with_node_identity() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let sk = ed25519_dalek::SigningKey::from_bytes(&[11u8; 32]);
        let id = "aira:identity:plane-signer";
        std::fs::create_dir_all(root.join("identity")).unwrap();
        std::fs::write(
            root.join("identity/local.ed25519"),
            format!("{}\n", hex::encode(sk.to_bytes())),
        )
        .unwrap();
        std::fs::write(
            root.join("identity/local.identity.json"),
            serde_json::json!({
                "identity_id": id,
                "identity_type": "local",
                "display_name": "plane-signer",
                "public_key": {
                    "algorithm": "ed25519",
                    "key_hex": hex::encode(sk.verifying_key().to_bytes())
                },
                "created_at": "2026-07-16T00:00:00Z",
                "key_path": "identity/local.ed25519"
            })
            .to_string(),
        )
        .unwrap();

        let mut session = LocalSession::open(&root).unwrap();
        assert_eq!(aira_object::primary_signer().as_str(), id);
        let out = session.submit_problem("Calculate 2 + 2").unwrap();
        assert!(matches!(out, SubmitOutcome::Completed { .. }));
        let problem = session.plane().problem_ref().unwrap();
        let desc = session
            .plane()
            .objects()
            .get_by_object_id(problem)
            .unwrap()
            .unwrap();
        assert_eq!(desc.producer_identity.as_str(), id);
        assert_eq!(desc.signature.key_ref.as_str(), id);
        desc.verify_canonical().unwrap();
        let ev = session
            .plane()
            .events()
            .iter()
            .find(|e| e.event_type == EventType::ProblemSubmitted)
            .unwrap();
        assert_eq!(ev.producer_identity.as_str(), id);
        assert_eq!(ev.signature.key_ref.as_str(), id);
        aira_object::reset_primary_signer();
    }

    fn write_default_yaml(root: &std::path::Path) {
        let yaml = serde_norway::to_string(&NodeConfig::default()).unwrap();
        std::fs::write(root.join("config.yaml"), yaml).unwrap();
    }

    #[test]
    fn load_config_yaml_matches_json() {
        let dir = tempfile::tempdir().unwrap();
        let root_json = dir.path().join("json");
        let root_yaml = dir.path().join("yaml");
        init_node(&root_json).unwrap();
        std::fs::create_dir_all(&root_yaml).unwrap();
        write_default_yaml(&root_yaml);
        let a = load_config(&root_json).unwrap();
        let b = load_config(&root_yaml).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn load_config_both_fail_closed() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        write_default_yaml(&root);
        let err = load_config(&root).unwrap_err().to_string();
        assert!(err.contains("both config.json and config.yaml"), "{err}");
    }

    #[test]
    fn load_config_json_only_ok() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        assert!(!root.join("config.yaml").exists());
        let cfg = load_config(&root).unwrap();
        assert_eq!(cfg.node.mode, "local");
    }

    #[test]
    fn open_accepts_yaml_only_node() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        // Layout without config.json: dirs + yaml.
        for d in ["identity", "db", "artifacts", "csu", "events", "problems"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        write_default_yaml(&root);
        assert!(node_config_present(&root));
        assert!(!root.join("config.json").exists());
        let session = LocalSession::open(&root).unwrap();
        assert_eq!(session.config.node.profile, "C1");
    }

    #[test]
    fn init_writes_json_not_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        assert!(root.join("config.json").exists());
        assert!(!root.join("config.yaml").exists());
    }

    #[test]
    fn init_idempotent_on_yaml_only_node() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        std::fs::create_dir_all(&root).unwrap();
        write_default_yaml(&root);
        init_node(&root).unwrap();
        assert!(root.join("config.yaml").exists());
        assert!(
            !root.join("config.json").exists(),
            "YAML-only node must not gain config.json from init"
        );
        let cfg = load_config(&root).unwrap();
        assert_eq!(cfg, NodeConfig::default());
    }

    #[test]
    fn status_accepts_yaml_only_node() {
        let _lock = isolated_flow();
        // `aira status` uses node_config_present + LocalSession::open.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        for d in ["identity", "db", "artifacts", "csu", "events", "problems"] {
            std::fs::create_dir_all(root.join(d)).unwrap();
        }
        write_default_yaml(&root);
        assert!(node_config_present(&root));
        let session = LocalSession::open(&root).unwrap();
        assert_eq!(session.config.node.mode, "local");
        assert_eq!(session.config.node.profile, "C1");
    }

    #[test]
    fn init_node_sqlite_object_path_migrate_and_persist() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let sqlite_path = root.join("db/aira.sqlite");
        assert!(sqlite_path.exists());

        let desc = aira_object::ObjectDescriptor::example_problem();
        let object_id = desc.object_id.clone();
        let mut store = aira_core::SqliteObjectStore::open(&sqlite_path).unwrap();
        store.create(desc.clone()).unwrap();
        drop(store);

        let reopened = aira_core::SqliteObjectStore::open(&sqlite_path).unwrap();
        let loaded = reopened.get_by_object_id(&object_id).unwrap().unwrap();
        assert_eq!(loaded, desc);

        let again = aira_core::SqliteObjectStore::open(&sqlite_path).unwrap();
        assert_eq!(again.get_by_object_id(&object_id).unwrap().unwrap(), desc);
    }

    #[test]
    fn plane_memory_beside_node_sqlite_object_path() {
        use aira_core::ObjectStore;

        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();

        let mut session = LocalSession::open(&root).unwrap();
        session.submit_problem("Calculate 2 + 2").unwrap();
        let problem = session.plane().problem_ref().unwrap().clone();
        assert!(
            session
                .plane()
                .objects()
                .get_by_object_id(&problem)
                .unwrap()
                .is_some(),
            "plane MemoryObjectStore must hold the submitted problem"
        );

        // Node SQLite path is independent of the plane memory store (#158).
        let mut sqlite = open_node_sqlite_object_store(&session.paths).unwrap();
        let mut durable = aira_object::ObjectDescriptor::example_problem();
        durable.object_id = aira_object::AiraRef::parse("aira:problem:01SQLITEBESIDE").unwrap();
        durable = durable
            .attach_canonical_signature()
            .expect("resign sqlite-only object");
        let durable_id = durable.object_id.clone();
        sqlite.create(durable.clone()).unwrap();
        drop(sqlite);

        let reopened = open_node_sqlite_object_store(&session.paths).unwrap();
        assert_eq!(
            reopened.get_by_object_id(&durable_id).unwrap().unwrap(),
            durable
        );
        // Plane memory still has the session problem; SQLite object is not auto-imported.
        assert!(session
            .plane()
            .objects()
            .get_by_object_id(&durable_id)
            .unwrap()
            .is_none());
        assert!(session
            .plane()
            .objects()
            .get_by_object_id(&problem)
            .unwrap()
            .is_some());
    }

    #[test]
    fn aira_core_manifest_has_no_node_or_peer_dep() {
        let core_toml = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../aira-core/Cargo.toml"),
        )
        .unwrap();
        for forbidden in ["aira-node", "aira-peer", "aira-flow", "aira-desktop"] {
            assert!(
                !core_toml.contains(forbidden),
                "aira-core must not depend on {forbidden} (Core↛node firewall)"
            );
        }
    }

    #[test]
    fn corrupt_event_log_recovered_and_writable() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let log_path = root.join("events/event-log.json");
        std::fs::write(&log_path, "{not-json").unwrap();

        let read = read_event_log_resilient(&log_path).unwrap();
        assert!(read.recovered_from_corruption);
        assert!(read.log.events.is_empty());
        assert!(root.join("events").join(EVENT_LOG_CORRUPT_BACKUP).exists());
        assert!(log_path.exists());

        let mut session = LocalSession::open(&root).unwrap();
        session.submit_problem("Calculate 2 + 2").unwrap();
        let tail = session.event_tail(20).unwrap();
        assert!(
            tail.iter()
                .any(|e| e.event_type == EventType::ProblemSubmitted),
            "events must append after corruption recovery"
        );
    }

    #[test]
    fn corrupt_trailing_event_log_recovers_valid_prefix() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let mut session = LocalSession::open(&root).unwrap();
        session.submit_problem("Calculate 2 + 2").unwrap();
        drop(session);

        let log_path = root.join("events/event-log.json");
        let good = std::fs::read_to_string(&log_path).unwrap();
        let before: EventLogFile = serde_json::from_str(&good).expect("valid log after submit");
        assert!(!before.events.is_empty(), "expected events after submit");
        let n = before.events.len();

        // Trailing junk after a complete JSON document (#155).
        std::fs::write(&log_path, format!("{good}\n,,,TRAILING_GARBAGE")).unwrap();
        let read = read_event_log_resilient(&log_path).unwrap();
        assert!(read.recovered_from_corruption);
        assert_eq!(read.log.events.len(), n);
        assert!(root.join("events").join(EVENT_LOG_CORRUPT_BACKUP).exists());

        // Truncated mid-array: keep first event object, destroy the rest (#155).
        let first = serde_json::to_string(&before.events[0]).unwrap();
        let truncated = format!("{{\"events\":[{first},{{this-is-not-json");
        std::fs::write(&log_path, &truncated).unwrap();
        let read2 = read_event_log_resilient(&log_path).unwrap();
        assert!(read2.recovered_from_corruption);
        assert_eq!(read2.log.events.len(), 1);
        assert_eq!(read2.log.events[0].event_id, before.events[0].event_id);
    }

    #[test]
    fn session_durable_file_chain_roundtrip() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".aira");
        init_node(&root).unwrap();
        let chain_path = root.join("events/file-chain-log.json");
        assert!(
            chain_path.exists(),
            "init_node must create file-chain durable log"
        );

        {
            let mut session = LocalSession::open(&root).unwrap();
            session.submit_problem("Calculate 2 + 2").unwrap();
            let tail = session.event_tail(50).unwrap();
            assert!(
                tail.iter()
                    .any(|e| e.event_type == EventType::ProblemSubmitted),
                "event_tail must read durable file-chain after submit"
            );
        }

        let durable = aira_event::FileChainEventLog::open(&chain_path).unwrap();
        assert!(!durable.is_empty());
        durable.chain().verify_tip().unwrap();
        assert!(durable
            .chain()
            .records()
            .iter()
            .any(|r| r.event.event_type == EventType::ProblemSubmitted));

        // Reopen session: durable events still visible via event_tail.
        let session = LocalSession::open(&root).unwrap();
        let tail = session.event_tail(50).unwrap();
        assert!(
            tail.iter()
                .any(|e| e.event_type == EventType::ProblemSubmitted),
            "reopened session must see durable file-chain events"
        );
    }

    #[test]
    fn research_artifact_rejected_as_operational_input() {
        let _lock = isolated_flow();
        let dir = tempfile::tempdir().unwrap();
        let mut plane = OperationalPlane::open(dir.path()).unwrap();
        let before = plane.events().len();

        let research_ev = make_event(
            "aira:event:research1",
            EventType::ResearchArtifactCreated,
            vec![],
            vec![],
            vec![],
            None,
        );
        let err = plane.inject_and_drain(research_ev).unwrap_err();
        assert!(
            matches!(err, FlowError::ResearchNonOperational(_)),
            "{err:?}"
        );
        assert_eq!(plane.events().len(), before);

        let promo_ev = make_event(
            "aira:event:promo1",
            EventType::ArtifactPromotionCandidate,
            vec![],
            vec![],
            vec![],
            None,
        );
        let err = plane.inject_and_drain(promo_ev).unwrap_err();
        assert!(matches!(err, FlowError::ResearchNonOperational(_)));
        assert_eq!(plane.events().len(), before);

        let payload = json_bytes(&json!({"kind": "research"}));
        let research_art = make_artifact(
            "aira:artifact:research-op1",
            ArtifactType::ResearchArtifact,
            &payload,
            vec![],
        );
        let art_id = research_art.artifact_id.clone();
        plane
            .artifacts_mut()
            .publish(research_art, &payload)
            .unwrap();
        let published = make_event(
            "aira:event:artpub-research1",
            EventType::ArtifactPublished,
            vec![],
            vec![art_id],
            vec![],
            None,
        );
        let err = plane.inject_and_drain(published).unwrap_err();
        assert!(matches!(err, FlowError::ResearchNonOperational(_)));
        assert_eq!(plane.events().len(), before);
    }
}
